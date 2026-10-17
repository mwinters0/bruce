//! Subcommands (D27). The only module that names a concrete profile, to build
//! the registry (D39).

use std::process::ExitCode;

use crate::profile::Registry;
use crate::profile::antora::Antora;
use crate::profile::generic::Generic;

/// The profiles compiled in, `generic` the fallback (D35).
pub fn registry() -> Registry {
    Registry::new(Box::new(Generic)).with(Box::new(Antora))
}

/// Runs the command line. No subcommand exists yet, so it says so and fails.
pub fn run() -> ExitCode {
    eprintln!(
        "bruce {}: no subcommand is implemented yet",
        env!("CARGO_PKG_VERSION")
    );
    ExitCode::FAILURE
}

#[cfg(test)]
mod tests {
    use std::process::ExitCode;

    use crate::config::ProfileChoice;
    use crate::store::{Mode, Store};

    use super::registry;

    /// `antora_pages`' columns.
    type Pivot = (
        u64,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    );

    #[test]
    fn a_run_with_no_subcommand_fails() {
        assert_eq!(super::run(), ExitCode::FAILURE);
    }

    #[test]
    fn auto_picks_antora_by_its_generator_and_generic_otherwise() {
        let r = registry();
        let pick = |g| r.choose(&ProfileChoice::Auto, g).unwrap().name();
        assert_eq!(pick(Some("Antora 3.1.15")), "antora");
        assert_eq!(pick(Some("Docusaurus v4.0.0")), "generic");
        assert_eq!(pick(None), "generic");
    }

    #[test]
    fn every_view_is_named_for_its_profile_and_creates() {
        let r = registry();
        for profile in r.iter() {
            for view in profile.views() {
                assert!(view.name.starts_with(&format!("{}_", profile.name())));
            }
        }
        Store::open_in_memory(&r).unwrap();
    }

    #[test]
    fn the_antora_view_pivots_a_pages_facts() {
        let store = Store::open_in_memory(&registry()).unwrap();
        let run = store.begin_run(Mode::Crawl, "abc", None).unwrap();
        let conn = store.connection();
        for (url_id, key, value) in [
            (1, "antora.component", "handbook"),
            (1, "antora.version", "f43"),
            (1, "antora.module", "ROOT"),
            (1, "antora.rel_path", "index.adoc"),
            (2, "antora.component", "arm-sbc"),
            (2, "antora.rel_path", "index.adoc"),
            (3, "other.component", "ignored"),
        ] {
            conn.execute(
                "INSERT INTO page_facts VALUES (?, ?, ?, ?)",
                duckdb::params![run.0, url_id, key, value],
            )
            .unwrap();
        }
        let mut stmt = conn
            .prepare(
                "SELECT url_id, component, version, module, rel_path \
                 FROM antora_pages ORDER BY url_id",
            )
            .unwrap();
        let rows: Vec<Pivot> = stmt
            .query_map([], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
            })
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        let s = |v: &str| Some(v.to_string());
        assert_eq!(
            rows,
            [
                (1, s("handbook"), s("f43"), s("ROOT"), s("index.adoc")),
                (2, s("arm-sbc"), None, None, s("index.adoc")),
            ]
        );
    }
}
