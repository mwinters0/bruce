//! The `antora_hints` view: for each unresolved xref (D20), the nearest page
//! ID among the run's pages (D65).
//!
//! The refspec is completed as Antora would have read it on the page it is on:
//! with no component it takes the page's, with only a module before the path
//! it takes the page's component, and `ROOT` is the empty module the IDs
//! below are written with (`component::path`). The IDs are the run's
//! `component:module:rel_path`, a version being none of them, and the nearest
//! is the least Levenshtein distance over the longer of the two, ties going to
//! the ID that sorts first. A hint is kept when that distance is at most 3 in
//! 10 and not 0, a page that is there being no hint for a refspec that did not
//! find it.

use super::super::View;

/// A row for each link whose verdict is `unresolved_xref` and for which a
/// page of the run lies within the distance: its run, page, raw refspec and
/// `hint`. A site without Antora pages has no row.
pub(super) const HINTS: View = View {
    name: "antora_hints",
    select: "\
WITH unresolved AS (
    SELECT v.run_id, v.src_url_id, v.xref_refspec, a.component, a.module,
        regexp_replace(regexp_replace(split_part(v.xref_refspec, '#', 1), '^[^@:$]*@', ''),
            '^page[$]', '') AS spec
    FROM link_verdicts v
    LEFT JOIN antora_pages a ON a.run_id = v.run_id AND a.url_id = v.src_url_id
    WHERE v.reason = 'unresolved_xref' AND v.xref_refspec IS NOT NULL
), wanted AS (
    SELECT run_id, src_url_id, xref_refspec, regexp_replace(
        CASE length(spec) - length(replace(spec, ':', ''))
            WHEN 0 THEN coalesce(component || ':' || CASE module WHEN 'ROOT' THEN '' ELSE module END || ':', '') || spec
            WHEN 1 THEN coalesce(component || ':', '') || spec
            ELSE spec
        END, '^([^:]*):ROOT:', '\\1::') AS want
    FROM unresolved
), known AS (
    SELECT DISTINCT run_id, component || ':' || CASE module WHEN 'ROOT' THEN '' ELSE module END || ':' || rel_path AS id
    FROM antora_pages
    WHERE component IS NOT NULL AND rel_path IS NOT NULL
), distances AS (
    SELECT w.run_id, w.want, k.id,
        levenshtein(w.want, k.id) AS edits, greatest(length(w.want), length(k.id)) AS longest
    FROM (SELECT DISTINCT run_id, want FROM wanted) w
    JOIN known k ON k.run_id = w.run_id
), nearest AS (
    SELECT run_id, want, arg_min(id, (edits::DOUBLE / longest, id)) AS hint,
        min(edits::DOUBLE / longest) AS ratio
    FROM distances
    GROUP BY run_id, want
)
SELECT DISTINCT w.run_id, w.src_url_id, w.xref_refspec, n.hint
FROM wanted w
JOIN nearest n ON n.run_id = w.run_id AND n.want = w.want
WHERE n.ratio > 0 AND n.ratio <= 0.3",
};
