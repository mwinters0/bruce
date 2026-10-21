//! What a signal does to a run `crawl` or `check-dir` is making (D58).
//!
//! SIGINT and SIGTERM are each a press: the first stops the run (D48), the
//! second abandons it (D49). A stopped or abandoned run exits 128 plus the
//! signal of the first press, as a shell reports a process a signal killed.

use std::future::Future;
use std::process::ExitCode;
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};

use tokio::runtime::Runtime;

use crate::crawl::{STOP_GRACE, Stopper, Writes};

/// The signal numbers whose status a shell reports as 128 plus the number.
const SIGINT: u8 = 2;
#[cfg(unix)]
const SIGTERM: u8 = 15;

/// The signal of the first press, which is the run's exit status less 128.
#[derive(Debug, Clone, Default)]
pub struct Pressed(Arc<AtomicU8>);

impl Pressed {
    /// 128 plus the first signal, or SIGINT's for a run no signal reached.
    pub fn exit(&self) -> ExitCode {
        match self.0.load(Ordering::SeqCst) {
            0 => ExitCode::from(128 + SIGINT),
            signal => ExitCode::from(128 + signal),
        }
    }
}

/// Runs `work` on `runtime` until it ends or a second signal abandons it, and
/// gives `None` for the abandoned. `stop` is what the first signal stops, and
/// `writes` is closed when the second comes, so the writes begun are
/// waited for and no others (D49). The runtime is shut down before this
/// returns, its tasks not waited for.
pub fn drive<T>(
    runtime: Runtime,
    command: &'static str,
    stop: Stopper,
    writes: &Writes,
    work: impl Future<Output = T>,
) -> (Option<T>, Pressed) {
    let pressed = Pressed::default();
    let out = runtime.block_on(async {
        tokio::select! {
            out = work => Some(out),
            () = watch(command, stop, &pressed) => None,
        }
    });
    runtime.shutdown_background();
    if out.is_none() {
        writes.close();
    }
    (out, pressed)
}

/// Stops the run at the first signal, and returns at the second.
async fn watch(command: &str, stop: Stopper, pressed: &Pressed) {
    let Ok(mut signals) = Signals::install() else {
        return std::future::pending().await;
    };
    let first = signals.next().await;
    pressed.0.store(first, Ordering::SeqCst);
    stop.stop();
    eprintln!(
        "bruce {command}: stopping: finishing requests in flight ({} s at most); Ctrl-C or \
         SIGTERM again to abandon them",
        STOP_GRACE.as_secs()
    );
    signals.next().await;
}

/// The two signals, listened for from [`Signals::install`] on.
#[cfg(unix)]
struct Signals(tokio::signal::unix::Signal, tokio::signal::unix::Signal);

#[cfg(unix)]
impl Signals {
    fn install() -> std::io::Result<Signals> {
        use tokio::signal::unix::{SignalKind, signal};
        Ok(Signals(
            signal(SignalKind::interrupt())?,
            signal(SignalKind::terminate())?,
        ))
    }

    /// The number of the next signal, or never where the runtime is gone.
    async fn next(&mut self) -> u8 {
        tokio::select! {
            Some(()) = self.0.recv() => SIGINT,
            Some(()) = self.1.recv() => SIGTERM,
            else => std::future::pending().await,
        }
    }
}

/// Where only Ctrl-C is a signal.
#[cfg(not(unix))]
struct Signals;

#[cfg(not(unix))]
impl Signals {
    fn install() -> std::io::Result<Signals> {
        Ok(Signals)
    }

    async fn next(&mut self) -> u8 {
        match tokio::signal::ctrl_c().await {
            Ok(()) => SIGINT,
            Err(_) => std::future::pending().await,
        }
    }
}
