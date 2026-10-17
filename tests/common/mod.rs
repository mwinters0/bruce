//! The saved pages under `fixtures/` (D24), shared by the fixture tests and
//! the bench.

use std::path::Path;

use url::Url;

/// One saved page.
pub struct Fixture {
    /// The path under `fixtures/`, `/`-separated.
    pub path: String,
    /// The URL the page is read as: its real one for docs.fp.o, where the
    /// path is the URL's, and a stand-in under `example.org` otherwise.
    pub url: Url,
    pub html: String,
}

fn walk(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            walk(&path, out);
        } else if path.extension().is_some_and(|e| e == "html") {
            out.push(path);
        }
    }
}

/// Every `.html` fixture, sorted by path.
pub fn fixtures() -> Vec<Fixture> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures");
    let mut files = Vec::new();
    walk(&root, &mut files);
    files.sort();
    files
        .into_iter()
        .map(|file| {
            let path = file
                .strip_prefix(&root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            let url = match path.strip_prefix("docs-fp-o/") {
                Some(rest) => format!(
                    "https://docs.fedoraproject.org/{}",
                    rest.strip_suffix("index.html").unwrap_or(rest)
                ),
                None => format!("https://example.org/{path}"),
            };
            Fixture {
                path,
                url: Url::parse(&url).unwrap(),
                html: std::fs::read_to_string(&file).unwrap(),
            }
        })
        .collect()
}
