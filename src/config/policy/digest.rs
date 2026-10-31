//! The fact the digest groups its pages by (D86): `[digest]` `group_by`, the
//! report's, recorded with the policy and judged by nothing.

use serde::Deserialize;

/// The key of the one row a run's `group_by` is recorded as (D78). The value is
/// empty for a site that groups by nothing, as every setting has a row.
pub(super) const KEY: &str = "digest.group_by";

/// A file's `[digest]` table (D86).
#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub(in crate::config) struct RawDigest {
    pub group_by: Option<String>,
}

/// The fact key a file writes: a profile's namespace, a dot and the key
/// (D37), as `antora.component`. Whether any page carries it is not checked,
/// the profiles that would record it not being known to the loader; a key no
/// page carries groups nothing.
pub(super) fn from_raw(written: Option<&str>) -> Result<Option<String>, String> {
    let Some(key) = written else {
        return Ok(None);
    };
    let part = |s: &str| {
        !s.is_empty()
            && s.chars()
                .all(|c| c.is_ascii_alphanumeric() || "_-".contains(c))
    };
    match key.split_once('.') {
        Some((namespace, rest)) if part(namespace) && rest.split('.').all(part) => {
            Ok(Some(key.to_owned()))
        }
        _ => Err(format!(
            "{key:?} is not a fact key: a profile's name, a dot and the fact, as antora.component \
             (leave group_by out to group by nothing)"
        )),
    }
}

/// The fact a run's `digest.group_by` row names; none where it is empty or
/// the run has no row.
pub fn from_row(value: Option<&str>) -> Option<String> {
    value.filter(|v| !v.is_empty()).map(str::to_owned)
}
