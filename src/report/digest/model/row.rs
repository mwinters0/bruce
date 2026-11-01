//! A table's row, its cells found by column name.

use crate::report::table::{Cell, Table};

/// One row of a table, its cells found by column name.
#[derive(Clone, Copy)]
pub(in crate::report::digest) struct Row<'a> {
    pub(in crate::report::digest) columns: &'a [String],
    pub(in crate::report::digest) cells: &'a [Cell],
}

impl<'a> Row<'a> {
    pub(in crate::report::digest) fn cell(&self, name: &str) -> &'a Cell {
        let at = self
            .columns
            .iter()
            .position(|c| c == name)
            .unwrap_or_else(|| panic!("the digest's table has no column `{name}`"));
        &self.cells[at]
    }

    pub(in crate::report::digest) fn text(&self, name: &str) -> Option<&'a str> {
        self.cell(name).text().filter(|s| !s.is_empty())
    }

    pub(in crate::report::digest) fn int(&self, name: &str) -> i64 {
        match self.cell(name) {
            Cell::Int(n) => *n,
            _ => 0,
        }
    }

    /// The row is new since the compared run (D75): as a finding under the
    /// severity map where the section has `new_listed`, the section's `new`
    /// where not.
    pub(in crate::report::digest) fn is_new(&self) -> bool {
        let column = if self.columns.iter().any(|c| c == "new_listed") {
            "new_listed"
        } else {
            "new"
        };
        matches!(self.cell(column), Cell::Bool(true))
    }

    pub(in crate::report::digest) fn list(&self, name: &str) -> &'a [String] {
        match self.cell(name) {
            Cell::List(items) => items,
            _ => &[],
        }
    }
}

pub(in crate::report::digest) fn rows(table: &Table) -> impl Iterator<Item = Row<'_>> {
    table.rows.iter().map(|cells| Row {
        columns: &table.columns,
        cells,
    })
}
