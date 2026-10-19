//! A report's rows and how they are printed: a text table, or one JSON array
//! of objects whose field names are the columns (D6).

use duckdb::types::Value;

/// One value of a report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Cell {
    Null,
    Text(String),
    Int(i64),
    Bool(bool),
}

impl From<Value> for Cell {
    /// The types the reports select; any other prints as its debug form, so a
    /// column added without a case here is visible rather than lost.
    fn from(value: Value) -> Cell {
        match value {
            Value::Null => Cell::Null,
            Value::Boolean(b) => Cell::Bool(b),
            Value::Text(s) => Cell::Text(s),
            Value::TinyInt(n) => Cell::Int(n.into()),
            Value::SmallInt(n) => Cell::Int(n.into()),
            Value::Int(n) => Cell::Int(n.into()),
            Value::BigInt(n) => Cell::Int(n),
            Value::UTinyInt(n) => Cell::Int(n.into()),
            Value::USmallInt(n) => Cell::Int(n.into()),
            Value::UInt(n) => Cell::Int(n.into()),
            Value::UBigInt(n) => {
                i64::try_from(n).map_or_else(|_| Cell::Text(n.to_string()), Cell::Int)
            }
            other => Cell::Text(format!("{other:?}")),
        }
    }
}

/// How a report is printed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Text,
    Json,
}

/// Named columns and rows of cells, in the order the report sorts them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<Cell>>,
}

impl Table {
    pub fn render(&self, format: Format) -> String {
        match format {
            Format::Text => self.text(),
            Format::Json => self.json(),
        }
    }

    /// A header and a row each, columns padded to the widest cell, numbers
    /// right-aligned, a missing value `-`. No row prints `(none)`.
    fn text(&self) -> String {
        let shown: Vec<Vec<String>> = self
            .rows
            .iter()
            .map(|row| row.iter().map(Cell::shown).collect())
            .collect();
        let widths: Vec<usize> = (0..self.columns.len())
            .map(|i| {
                shown
                    .iter()
                    .map(|r| r[i].chars().count())
                    .chain([self.columns[i].chars().count()])
                    .max()
                    .unwrap_or(0)
            })
            .collect();
        let right: Vec<bool> = (0..self.columns.len())
            .map(|i| self.rows.iter().any(|r| matches!(r[i], Cell::Int(_))))
            .collect();
        let line = |cells: &[String]| {
            let padded: Vec<String> = cells
                .iter()
                .enumerate()
                .map(|(i, c)| {
                    let pad = " ".repeat(widths[i] - c.chars().count());
                    if right[i] {
                        format!("{pad}{c}")
                    } else {
                        format!("{c}{pad}")
                    }
                })
                .collect();
            padded.join("  ").trim_end().to_owned()
        };
        let mut out = line(&self.columns);
        for row in &shown {
            out.push('\n');
            out.push_str(&line(row));
        }
        if self.rows.is_empty() {
            out.push_str("\n(none)");
        }
        out
    }

    /// One array of objects, a row each, its fields in column order.
    fn json(&self) -> String {
        let rows: Vec<String> = self
            .rows
            .iter()
            .map(|row| {
                let fields: Vec<String> = self
                    .columns
                    .iter()
                    .zip(row)
                    .map(|(name, cell)| format!("{}:{}", quoted(name), cell.json()))
                    .collect();
                format!("{{{}}}", fields.join(","))
            })
            .collect();
        format!("[{}]", rows.join(",\n "))
    }
}

impl Cell {
    fn shown(&self) -> String {
        match self {
            Cell::Null => "-".to_owned(),
            Cell::Text(s) => s.clone(),
            Cell::Int(n) => n.to_string(),
            Cell::Bool(b) => b.to_string(),
        }
    }

    fn json(&self) -> String {
        match self {
            Cell::Null => "null".to_owned(),
            Cell::Text(s) => quoted(s),
            Cell::Int(n) => n.to_string(),
            Cell::Bool(b) => b.to_string(),
        }
    }
}

/// A JSON string: the quote, the backslash and every control character
/// escaped, everything else as it is.
fn quoted(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c < ' ' => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests;
