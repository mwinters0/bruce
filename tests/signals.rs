//! What SIGINT and SIGTERM do to `bruce crawl` and `bruce check-dir` (D58),
//! over the built binary: a signal is process-wide, so it cannot be raised in
//! the test process beside the tests that run a crawl.
//!
//! The server answers `/robots.txt` after a delay and every other path with a
//! 404 at once. Both commands ask for it before anything else of the host, so a
//! signal sent once it is asked for lands while the run is in flight.

#![cfg(unix)]

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;

use bruce::cli::registry;
use bruce::store::{Mode, Store};

/// How long a test waits for what it expects, against a hung binary.
const PATIENCE: Duration = Duration::from_secs(20);

/// A server at the returned address, and a channel that hears each time
/// `/robots.txt` is asked for.
fn server(delay: Duration) -> (String, Receiver<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let (asked, heard) = mpsc::channel();
    thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let asked = asked.clone();
            thread::spawn(move || answer(stream, delay, &asked));
        }
    });
    (format!("127.0.0.1:{}", addr.port()), heard)
}

fn answer(mut stream: TcpStream, delay: Duration, asked: &mpsc::Sender<()>) {
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut first = String::new();
    if reader.read_line(&mut first).is_err() {
        return;
    }
    if first.contains(" /robots.txt ") {
        let _ = asked.send(());
        thread::sleep(delay);
    }
    let _ = stream
        .write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
}

/// A running `bruce`, its stderr read line by line.
struct Bruce {
    child: Child,
    stderr: Receiver<String>,
}

impl Bruce {
    fn spawn(args: &[&str]) -> Bruce {
        let mut child = Command::new(env!("CARGO_BIN_EXE_bruce"))
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let stderr = child.stderr.take().unwrap();
        let (lines, heard) = mpsc::channel();
        thread::spawn(move || {
            for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                let _ = lines.send(line);
            }
        });
        Bruce {
            child,
            stderr: heard,
        }
    }

    /// Sends `signal` (`TERM`, `INT`) to the process.
    fn signal(&self, signal: &str) {
        let sent = Command::new("kill")
            .arg(format!("-{signal}"))
            .arg(self.child.id().to_string())
            .status()
            .unwrap();
        assert!(sent.success());
    }

    /// Waits for a line of stderr holding `needle`.
    fn hears(&self, needle: &str) {
        while let Ok(line) = self.stderr.recv_timeout(PATIENCE) {
            if line.contains(needle) {
                return;
            }
        }
        panic!("no line of stderr held {needle:?}");
    }

    /// What the run printed to stdout, the rest of stderr, and how it ended.
    fn end(mut self) -> (String, String, ExitStatus) {
        let mut out = String::new();
        self.child
            .stdout
            .take()
            .unwrap()
            .read_to_string(&mut out)
            .unwrap();
        let status = self.child.wait().unwrap();
        let err: Vec<String> = self.stderr.try_iter().collect();
        (out, err.join("\n"), status)
    }
}

/// `bruce crawl` of the server, signalled once it asks for `/robots.txt`.
fn crawl(delay: Duration, db: &Path) -> (Bruce, Receiver<()>) {
    let (host, asked) = server(delay);
    let bruce = Bruce::spawn(&[
        "crawl",
        &format!("http://{host}/"),
        "--db",
        db.to_str().unwrap(),
    ]);
    (bruce, asked)
}

/// `bruce check-dir` of a one-page build linking to the server, signalled
/// once it asks for `/robots.txt`. The baseline is an empty finished run.
fn check_dir(delay: Duration, tmp: &Path) -> (Bruce, Receiver<()>) {
    let (host, asked) = server(delay);
    let build = tmp.join("build");
    std::fs::create_dir_all(&build).unwrap();
    std::fs::write(
        build.join("index.html"),
        format!(r#"<a href="http://{}/x">x</a>"#, host),
    )
    .unwrap();
    let config = tmp.join("bruce.toml");
    std::fs::write(
        &config,
        "allowlist = [\"127.0.0.1\"]\n[[root]]\nurl = \"https://x.test/docs/\"\n",
    )
    .unwrap();
    let baseline = tmp.join("baseline.duckdb");
    let store = Store::open(&baseline, &registry()).unwrap();
    let run = store.begin_run(Mode::Crawl, "base", None, &[]).unwrap();
    store.finish_run(run).unwrap();
    drop(store);
    let bruce = Bruce::spawn(&[
        "check-dir",
        build.to_str().unwrap(),
        "--prefix",
        "https://x.test/docs/",
        "--baseline",
        baseline.to_str().unwrap(),
        "--config",
        config.to_str().unwrap(),
        "--externals",
        "live",
        "--db",
        tmp.join("bruce.duckdb").to_str().unwrap(),
    ]);
    (bruce, asked)
}

/// Whether run `run` of the database at `db` is finished.
fn finished(db: &Path, run: u64) -> bool {
    let store = Store::open(db, &registry()).unwrap();
    store
        .connection()
        .query_row(
            "SELECT finished_at IS NOT NULL FROM runs WHERE run_id = ?",
            [run],
            |row| row.get(0),
        )
        .unwrap()
}

#[test]
fn a_crawl_stopped_by_sigterm_leaves_its_run_unfinished_and_exits_143() {
    let tmp = tempfile::tempdir().unwrap();
    let db = tmp.path().join("bruce.duckdb");
    let (bruce, asked) = crawl(Duration::from_secs(1), &db);
    asked.recv_timeout(PATIENCE).unwrap();
    bruce.signal("TERM");
    let (out, err, status) = bruce.end();
    assert_eq!(status.code(), Some(143), "{err}");
    assert!(err.contains("stopping"), "{err}");
    assert!(out.contains("stopped before the end"), "{out}");
    assert!(!finished(&db, 1));
}

#[test]
fn a_crawl_stopped_by_sigint_exits_130() {
    let tmp = tempfile::tempdir().unwrap();
    let (bruce, asked) = crawl(Duration::from_secs(1), &tmp.path().join("bruce.duckdb"));
    asked.recv_timeout(PATIENCE).unwrap();
    bruce.signal("INT");
    let (_, err, status) = bruce.end();
    assert_eq!(status.code(), Some(130), "{err}");
}

#[test]
fn a_second_signal_abandons_a_crawl_and_the_first_decides_the_status() {
    let tmp = tempfile::tempdir().unwrap();
    let db = tmp.path().join("bruce.duckdb");
    let (bruce, asked) = crawl(Duration::from_secs(8), &db);
    asked.recv_timeout(PATIENCE).unwrap();
    bruce.signal("TERM");
    bruce.hears("stopping");
    bruce.signal("INT");
    let (out, err, status) = bruce.end();
    assert_eq!(status.code(), Some(143), "{err}");
    assert!(out.contains("abandoned at the second signal"), "{out}");
    assert!(!finished(&db, 1));
}

#[test]
fn a_stopped_check_dir_judges_nothing_and_leaves_its_run_unfinished() {
    let tmp = tempfile::tempdir().unwrap();
    let (bruce, asked) = check_dir(Duration::from_secs(1), tmp.path());
    asked.recv_timeout(PATIENCE).unwrap();
    bruce.signal("TERM");
    let (out, err, status) = bruce.end();
    assert_eq!(status.code(), Some(143), "{err}");
    assert!(err.contains("the build was not judged"), "{err}");
    assert!(err.contains("stopped before the end"), "{err}");
    assert_eq!(out, "");
    // Run 1 is the imported baseline and run 2 the check.
    assert!(finished(&tmp.path().join("bruce.duckdb"), 1));
    assert!(!finished(&tmp.path().join("bruce.duckdb"), 2));
}

#[test]
fn a_second_signal_abandons_a_check_dir_unjudged() {
    let tmp = tempfile::tempdir().unwrap();
    let (bruce, asked) = check_dir(Duration::from_secs(8), tmp.path());
    asked.recv_timeout(PATIENCE).unwrap();
    bruce.signal("INT");
    bruce.hears("stopping");
    bruce.signal("TERM");
    let (out, err, status) = bruce.end();
    assert_eq!(status.code(), Some(130), "{err}");
    assert!(err.contains("abandoned at the second signal"), "{err}");
    assert_eq!(out, "");
    assert!(!finished(&tmp.path().join("bruce.duckdb"), 2));
}
