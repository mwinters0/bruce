//! The `changed` report: the pages whose content hash moved between two runs
//! (D15).
//!
//! A hash is comparable only within one scope and one root's keys, so a page
//! is `changed` where both runs hashed it under the same `hash_scope` and the
//! same `hash_root` and `hash_exclude`, and its hash differs; where either
//! differs it is `scope_changed`, whatever its hash did. Only pages a root
//! covers (`pages.root`) are read, the forge's and the other allowlisted
//! pages being hashed under no root's keys. A page that is `whole` under a
//! root that sets `hash_root` is one that root's region missed, and is listed
//! with `root_missed` set whether or not its hash moved, the
//! misconfiguration a page changing on every build shows. That is a fact of
//! `to` alone, so the page need not be in `from`.

use duckdb::Connection;

use super::error::Error;
use super::request::Request;
use super::sql::Sql;
use super::table::Table;
use crate::store::RunId;

/// Every run's roots with the keys their hashes were made under.
const KEYS: &str = "WITH keys AS (SELECT run_id, s.root AS root, s.hash_root AS hash_root, \
                    s.hash_exclude AS hash_exclude \
                    FROM (SELECT run_id, unnest(hash_scopes) AS s FROM runs)) ";

/// A row per page of `to` that changed, whose scope changed or that its root
/// missed: `changed` first, then `scope_changed`, then `root_missed`, by page.
/// `--repo` and `--fact` filter the page in `to`.
pub(super) fn changed(
    conn: &Connection,
    from: RunId,
    to: RunId,
    request: &Request,
) -> Result<(Table, Vec<String>), Error> {
    let mut sql = Sql::new(KEYS);
    sql.push(
        "SELECT CASE WHEN scope_changed THEN 'scope_changed' WHEN moved THEN 'changed' \
         ELSE 'root_missed' END AS change, page, scope_from, scope_to, root_missed, title, \
         source_repo, source_path FROM (\
         SELECT t.url AS page, a.hash_scope AS scope_from, b.hash_scope AS scope_to, \
         coalesce(a.root IS NOT NULL AND (a.hash_scope <> b.hash_scope OR a.root <> b.root \
         OR ka.hash_root IS DISTINCT FROM kb.hash_root \
         OR ka.hash_exclude IS DISTINCT FROM kb.hash_exclude), false) AS scope_changed, \
         coalesce(a.root IS NOT NULL AND a.content_hash IS DISTINCT FROM b.content_hash, false) \
         AS moved, \
         coalesce(b.hash_scope = 'whole' AND len(kb.hash_root) > 0, false) AS root_missed, \
         b.title, b.source_repo, b.source_path \
         FROM pages b \
         JOIN targets t ON t.run_id = b.run_id AND t.url_id = b.url_id \
         LEFT JOIN pages a ON a.url_id = b.url_id AND a.run_id = ",
    )
    .bind(from.0)
    .push(
        " LEFT JOIN keys ka ON ka.run_id = a.run_id AND ka.root = a.root \
         LEFT JOIN keys kb ON kb.run_id = b.run_id AND kb.root = b.root \
         WHERE b.root IS NOT NULL AND ",
    );
    sql.scope(
        to,
        request,
        super::sql::Cols {
            run: "b.run_id",
            repo: "b.source_repo",
            page: "b.url_id",
        },
    );
    sql.push(
        ") WHERE scope_changed OR moved OR root_missed \
         ORDER BY change = 'root_missed', change = 'scope_changed', page",
    )
    .limit(request);
    let note = format!("from run {} to run {}", from.0, to.0);
    Ok((sql.fetch(conn)?, vec![note]))
}

#[cfg(test)]
mod tests;
