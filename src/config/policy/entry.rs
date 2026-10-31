//! One row of a run's recorded policy (D78), and how a person reads it.

/// One row of a run's recorded policy: `host` is `None` for the run-wide
/// value (D78).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub host: Option<String>,
    pub key: String,
    pub value: String,
}

impl Entry {
    /// The setting as a file writes it, for a person to read: its dotted
    /// path, then `=` and the value, `+=` for an element of `judge.schemes`.
    pub fn written_as(&self) -> String {
        if self.host.is_none() && self.key == "gate.fail" {
            let list: Vec<String> = self.value.split(',').map(|o| format!("{o:?}")).collect();
            return format!("gate.fail = [{}]", list.join(", "));
        }
        let value = if matches!(self.value.as_str(), "true" | "false") {
            self.value.clone()
        } else {
            format!("{:?}", self.value)
        };
        let (path, op) = match (self.host.as_deref(), self.key.as_str()) {
            (Some(host), key) => (format!("host.{host:?}.{key}"), "="),
            (None, "schemes") => ("judge.schemes".to_owned(), "+="),
            (None, key @ ("pedantic_trailing_slash" | "doubled_slash_reach")) => {
                (format!("judge.{key}"), "=")
            }
            (None, key) if key.starts_with("severity.") => (key.to_owned(), "="),
            (None, key) => (format!("default.{key}"), "="),
        };
        format!("{path} {op} {value}")
    }
}
