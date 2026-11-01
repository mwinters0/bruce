//! The version of the model a view says it was written for (D96): the integer
//! in its `{% set bruce_model = N %}`.

use regex::Regex;

use crate::report::digest::FORMAT_VERSION;

/// Where the manual explains the declaration, named in the refusal.
const MANUAL: &str = "docs/manual/digest.md, \"The model's version\"";

/// `source`'s declaration refused where it names another version than this
/// `bruce`'s model, or is not a whole number; a source with none passes. It is
/// read off the text, so a view the model has outgrown is refused for its
/// version and not for the first field it names that is gone.
pub(super) fn check(source: &str) -> Result<(), String> {
    let tag = Regex::new(r"\{%[-+]?\s*set\s+bruce_model\s*=\s*([^%]*?)\s*[-+]?%\}")
        .expect("the pattern is valid");
    let Some(found) = tag.captures(source) else {
        return Ok(());
    };
    match found[1].parse::<u32>() {
        Ok(FORMAT_VERSION) => Ok(()),
        Ok(other) => Err(format!(
            "it declares model version {other} and this bruce's is {FORMAT_VERSION}: see {MANUAL}"
        )),
        Err(_) => Err(format!(
            "`bruce_model` is set to `{}`, which is no whole number: see {MANUAL}",
            &found[1]
        )),
    }
}
