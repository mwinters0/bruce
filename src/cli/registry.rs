//! The registry `cli` builds (D39): the profiles compiled in, and those a
//! config defines ahead of them (D35).

use crate::config::Definition;
use crate::profile::antora::Antora;
use crate::profile::declarative::Declarative;
use crate::profile::generic::Generic;
use crate::profile::{Profile, Registry};

/// The profiles compiled in, `generic` the fallback (D35).
pub fn registry() -> Registry {
    Registry::new(Box::new(Generic)).with(Box::new(Antora))
}

/// The compiled profile called `name`, which a declared one may extend (D94).
fn compiled(name: &str) -> Option<Box<dyn Profile>> {
    match name {
        "generic" => Some(Box::new(Generic)),
        "antora" => Some(Box::new(Antora)),
        _ => None,
    }
}

/// [`registry`] with the profiles a config defines, which `auto` tries before
/// the compiled ones, in the file's order (D35). `Err` for a profile named as
/// a compiled one, one extending what is not compiled in, or one that cannot
/// be built.
pub fn registry_for(defined: &[Definition]) -> Result<Registry, String> {
    let mut registry = Registry::new(Box::new(Generic));
    for definition in defined {
        let name = definition.name.as_str();
        if matches!(name, "generic" | "antora") {
            return Err(format!(
                "profile.{name}: `{name}` is compiled in; a profile defined here takes a name \
                 of its own"
            ));
        }
        let base = match definition.extends.as_deref() {
            None => None,
            Some(base) => Some(compiled(base).ok_or_else(|| {
                format!(
                    "profile.{name}: extends: `{base}` is not a compiled profile (`generic` or \
                     `antora`)"
                )
            })?),
        };
        let built =
            Declarative::new(definition, base).map_err(|e| format!("profile.{name}: {e}"))?;
        registry = registry.with(Box::new(built));
    }
    Ok(registry.with(Box::new(Antora)))
}
