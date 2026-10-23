//! The DuckDB schema (D26 as amended by D15, D28, D34, D35, D36, D37, D43 and D51) and
//! its version. Rows are append-only (D2), so only `runs.finished_at` is ever
//! updated and only `Store::prune` deletes; there are no primary keys on the
//! bulk tables, which an appender would pay to check for nothing.

/// The version this build writes and reads (D43). A schema change bumps it;
/// a 0.x release migrates no database (D43).
pub const SCHEMA_VERSION: u32 = 5;

/// The `meta` key holding [`SCHEMA_VERSION`].
pub const VERSION_KEY: &str = "schema_version";

/// The five outcomes of D16.
const OUTCOME: &str = "('ok', 'broken_url', 'broken_anchor', 'error', 'unchecked')";

/// A run root's hash scope (D15): the root's canonical URL, as `runs.roots`
/// holds it, and its `hash_root` and `hash_exclude`.
pub(super) const SCOPE: &str = "STRUCT(root VARCHAR, hash_root VARCHAR[], hash_exclude VARCHAR[])";

/// What a page's hash covered (D15).
const HASH_SCOPE: &str = "('root', 'region', 'whole')";

/// Where a target's answer came from (D51).
const ANSWERED_BY: &str = "('local', 'baseline', 'live')";

/// Every table, created together in one transaction on a new file. A URL's
/// `url_id` is the u64 hash of its canonical form (D26), so it joins across
/// runs without comparing URLs; a target with reason `invalid_url` has its
/// raw href in place of a form.
pub fn create() -> String {
    format!(
        "BEGIN;
CREATE TABLE meta (key VARCHAR PRIMARY KEY, value VARCHAR NOT NULL);
CREATE TABLE runs (
    run_id UBIGINT PRIMARY KEY,
    started_at TIMESTAMPTZ NOT NULL,
    finished_at TIMESTAMPTZ,
    mode VARCHAR NOT NULL CHECK (mode IN ('crawl', 'local')),
    tool_version VARCHAR NOT NULL,
    config_hash VARCHAR NOT NULL,
    baseline_run UBIGINT,
    roots VARCHAR[] NOT NULL,
    hash_scopes {SCOPE}[] NOT NULL
);
CREATE TABLE targets (
    run_id UBIGINT NOT NULL,
    url_id UBIGINT NOT NULL,
    url VARCHAR NOT NULL,
    status USMALLINT,
    final_url VARCHAR,
    redirect_chain STRUCT(url VARCHAR, status USMALLINT, refresh BOOLEAN)[],
    outcome VARCHAR NOT NULL CHECK (outcome IN {OUTCOME}),
    reason VARCHAR,
    checked_at TIMESTAMPTZ,
    duration_ms UINTEGER,
    answered_by VARCHAR NOT NULL CHECK (answered_by IN {ANSWERED_BY})
);
CREATE TABLE pages (
    run_id UBIGINT NOT NULL,
    url_id UBIGINT NOT NULL,
    title VARCHAR,
    content_type VARCHAR,
    size UBIGINT,
    content_hash BLOB,
    hash_scope VARCHAR NOT NULL CHECK (hash_scope IN {HASH_SCOPE}),
    profile VARCHAR NOT NULL,
    source_url VARCHAR,
    source_repo VARCHAR,
    source_ref VARCHAR,
    source_path VARCHAR,
    in_sitemap BOOLEAN NOT NULL,
    via_links BOOLEAN NOT NULL,
    root VARCHAR
);
CREATE TABLE page_facts (
    run_id UBIGINT NOT NULL,
    url_id UBIGINT NOT NULL,
    key VARCHAR NOT NULL,
    value VARCHAR NOT NULL
);
CREATE TABLE anchors (
    run_id UBIGINT NOT NULL,
    url_id UBIGINT NOT NULL,
    anchor VARCHAR NOT NULL,
    tag VARCHAR NOT NULL,
    placement VARCHAR NOT NULL CHECK (placement IN ('content', 'chrome', 'unknown'))
);
CREATE TABLE links (
    run_id UBIGINT NOT NULL,
    src_url_id UBIGINT NOT NULL,
    target_url_id UBIGINT NOT NULL,
    fragment VARCHAR,
    raw_href VARCHAR NOT NULL,
    kind VARCHAR NOT NULL,
    xref_refspec VARCHAR
);
CREATE TABLE bodies (
    run_id UBIGINT NOT NULL,
    url_id UBIGINT NOT NULL,
    html BLOB NOT NULL
);
COMMIT;"
    )
}
