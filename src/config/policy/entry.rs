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
    /// Whether the row is part of a declarative profile's definition (D78),
    /// which a run extracted by and no report judges by.
    pub fn is_profile(&self) -> bool {
        self.host.is_none() && self.key.starts_with("profile.")
    }

    /// The setting as a file writes it, for a person to read: its dotted
    /// path, then `=` and the value, `+=` for an element of `judge.schemes`.
    pub fn written_as(&self) -> String {
        if self.host.is_none() && self.key == "gate.fail" {
            let list: Vec<String> = self
                .value
                .split(',')
                .filter(|o| !o.is_empty())
                .map(|o| format!("{o:?}"))
                .collect();
            return format!("gate.fail = [{}]", list.join(", "));
        }
        let value = if matches!(self.value.as_str(), "true" | "false")
            || self.value.parse::<u64>().is_ok()
        {
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
            (None, key) if key.starts_with("profile.") => (
                key.to_owned(),
                if key.ends_with(".region") { "+=" } else { "=" },
            ),
            (None, key)
                if key.starts_with("severity.")
                    || matches!(key, "max_body_mib" | "digest.group_by") =>
            {
                (key.to_owned(), "=")
            }
            (None, key) => (format!("default.{key}"), "="),
        };
        format!("{path} {op} {value}")
    }
}
