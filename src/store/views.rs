//! The views the store ships, installed at every open beside the profiles'
//! (D6, D28).
//!
//! `link_verdicts` is a link's verdict, computed whenever it is read (D28): the
//! link's outcome, reason and whether its target was redirected, read off
//! `links`, its target, the target's landing target and the landing page's
//! `anchors`. A column holds the link's own fields, so a report reads one
//! relation. It is the call, naming no run, of the table macro
//! `link_verdicts_with` (D77, D78), judging by D81's `judged_by` where it is
//! set and else by the link's own run's policy, which reads the `policy_in_force`
//! view (D84). The target's answer is first read
//! through the policy: a status code the policy judges, a scheme it counts
//! unchecked or not. The first rule that applies then decides:
//!
//! 1. A link with `xref_refspec` that points at its own page, whose
//!    fragment is the refspec, or whose target is `broken_url`, is
//!    `broken_url`, reason `unresolved_xref`, whatever its target answered
//!    (D20). A CI run that resolved the refspec points the link at the page it
//!    names with the refspec's own fragment, and keeps the refspec: rules 2 to
//!    8 judge it (D55), unless the URL it names, which Resolve may only have
//!    derived, is itself broken (D57).
//! 2. A link whose target has no row, which only a stopped run leaves, is
//!    `unchecked`, reason `cut`.
//! 3. A link to a URL whose path doubles a slash, or to one that redirected
//!    to such a URL, is `broken_url`, reason `doubled_slash`, whatever the
//!    target answered and asked or not, bar a page's link to itself, a
//!    `#fragment` link on a doubled-slash page being that page's (D71), and
//!    unless the policy turns it off for the host of the URL linked.
//! 4. A target that is not `ok` gives its link its own outcome and reason: the
//!    fragment is not looked at (D16).
//! 5. No fragment, an empty one, and `top` in any case are `ok` (D12). `top`
//!    is a browser's own and HTML matches it without regard to case.
//! 6. A text fragment (`:~:`) is `unchecked`, reason `text_fragment` (D12).
//! 7. An `ok` target with no page to read (`too_large`, an asset, an
//!    allowlisted URL first named by a non-anchor link) is `unchecked`,
//!    reason `no_anchors`: nothing is known, so nothing is broken.
//! 8. Otherwise the fragment is checked against the anchors of the page the
//!    target landed on (D46): `ok` where an anchor is the fragment as written
//!    or percent-decoded, as a browser looks (D12), else `broken_anchor`.
//!
//! An `ok` link whose target was redirected to its own URL with a final `/`
//! added or removed has the reason `pedantic_trailing_slash`, whatever the
//! hops between (D70), unless the policy turns it off. It is not an outcome of
//! its own.
//!
//! `relative` says the link's `raw_href` keeps its page's path: it is resolved
//! against it (`../`, `committee/`) or is a `?query` on it. `counted` says the
//! link counts in the reports (D88): not a link on a page whose URL doubles a
//! slash and whose twin, the URL with its slashes collapsed, is a page of the
//! run, and not a broken or errored `relative` one on such a page without a twin, which
//! the doubled slash breaks and the linking page answers for. `doubled_slash`
//! lists the targets whose URL doubles a slash, or whose landing does, with
//! their `url`, and for the first the page of the run at its twin's URL,
//! `twin_id`, whatever the policy says: the macro applies it.

use duckdb::Connection;

use crate::config::WORKING_SCHEMES;
use crate::profile::{Registry, View};

/// Every run read through `link_verdicts` is judged by one run's policy while
/// this lives: the run a comparison reports on, in one (D81). Dropping it puts each
/// run back to its own. It sets the connection's session variable `judged_by`,
/// which the macro reads where its `policy_run` is NULL, so the views, which
/// take no parameter (D6), judge both runs of a comparison by one's policy.
#[must_use = "the policy judges only while the guard lives"]
pub struct JudgedBy<'c> {
    conn: &'c Connection,
}

impl<'c> JudgedBy<'c> {
    /// Judges every run by the policy of run number `run`.
    pub fn new(conn: &'c Connection, run: u64) -> Result<JudgedBy<'c>, duckdb::Error> {
        conn.execute_batch(&format!("SET VARIABLE judged_by = {run}"))?;
        Ok(JudgedBy { conn })
    }
}

impl Drop for JudgedBy<'_> {
    fn drop(&mut self) {
        // A connection that cannot say so is gone, and its variable with it.
        let _ = self.conn.execute_batch("SET VARIABLE judged_by = NULL");
    }
}

/// The `doubled_slash` view, created before the macro that joins it.
pub(super) const FIRST: &[View] = &[POLICY_IN_FORCE, DOUBLED_SLASH_TARGETS];

/// The policy rows the views and the reports read (D78): the stored `policy`,
/// unless [`Store::try_policy`] has put a trial's rows in its place (D84).
const POLICY_IN_FORCE: View = View {
    name: "policy_in_force",
    select: "SELECT * FROM policy",
};

/// The `link_hints` view, created after the profiles' views it reads: the
/// hint of every unresolved xref that a profile's `<profile>_hints` view
/// offers (D87), the columns of such a view and no row where no profile ships
/// one. A key two profiles hint keeps the least hint, so a link is one row.
pub(super) fn link_hints(profiles: &Registry) -> String {
    let columns = "run_id, src_url_id, xref_refspec, hint";
    let shipped: Vec<String> = profiles
        .iter()
        .filter_map(|p| {
            let name = format!("{}_hints", p.name());
            let ships = p.views().iter().any(|v| v.name == name);
            ships.then(|| format!("SELECT {columns} FROM {name}"))
        })
        .collect();
    let select = match shipped.as_slice() {
        [] => "SELECT run_id, src_url_id, xref_refspec, NULL::VARCHAR AS hint FROM links \
               WHERE false"
            .to_owned(),
        [one] => one.clone(),
        many => format!(
            "SELECT run_id, src_url_id, xref_refspec, min(hint) AS hint FROM ({}) \
             GROUP BY run_id, src_url_id, xref_refspec",
            many.join(" UNION ALL ")
        ),
    };
    format!("CREATE OR REPLACE VIEW link_hints AS {select}")
}

/// The view calling the macro, created after it.
pub(super) const LAST: &[View] = &[LINK_VERDICTS];

/// A true expression where the path of the `http(s)` URL `$url` doubles a
/// slash. A `scheme://` inside the path doubles none, nor does a query or a
/// fragment, and nor does the body of a `data:` URL or a `mailto:` (D71). The
/// cheap `contains` goes first, a conjunction being tried in order.
macro_rules! doubled {
    ($url:literal) => {
        concat!(
            "coalesce(contains(substr(",
            $url,
            ", 8), '//') AND regexp_matches(regexp_extract(",
            $url,
            ", '^https?://[^?#]*'), '[^:/]//'), false)"
        )
    };
}

/// A true expression where the URL of the target aliased `t` doubles a slash,
/// for a query telling a `doubled_slash` target's own doubled slash from its
/// landing's without the `doubled_slash` view's scan of every target.
pub const TARGET_URL_DOUBLES: &str = doubled!("t.url");

/// The targets whose URL doubles a slash (`own`) or that redirected to a URL
/// that does, with their `url`, and for the first its `twin_id`, the page of
/// the run at its URL with the slashes collapsed, if any (D88). It joins the
/// few, not every target of the run, to look for the twin.
const DOUBLED_SLASH_TARGETS: View = View {
    name: "doubled_slash",
    select: concat!(
        "\
SELECT d.run_id, d.url_id, d.url, d.own, min(tp.url_id) AS twin_id
FROM (SELECT run_id, url_id, url, ",
        doubled!("url"),
        " AS own FROM targets WHERE ",
        doubled!("url"),
        " OR ",
        doubled!("final_url"),
        ") d
LEFT JOIN targets tt ON d.own AND tt.run_id = d.run_id
    AND tt.url = regexp_replace(d.url, '([^:/])/{2,}', '\\1/', 'g')
LEFT JOIN pages tp ON tp.run_id = tt.run_id AND tp.url_id = tt.url_id
GROUP BY d.run_id, d.url_id, d.url, d.own"
    ),
};

/// The host of the URL `$url`, lower case, as a policy row's `host` names it.
macro_rules! host {
    ($url:literal) => {
        concat!(
            "lower(regexp_extract(",
            $url,
            r", '^https?://(?:[^/?#@]*@)?(\[[^\]]*\]|[^/?#:]*)', 1))"
        )
    };
}

/// The text of the table macro judging every link under a policy (D77, D78),
/// the body of `link_verdicts` (D28). `policy_run` names the run whose
/// recorded policy judges, else the one [`JudgedBy`] names (D81), else the
/// link's own; a setting with no
/// row is at its default, so a run recorded before the setting existed, or by
/// a test, is judged as `bruce` always has been. The landing target is the one
/// `final_url` names, and a target that was not redirected is its own landing
/// page.
///
/// A target is read as `answer`, an outcome and a reason: its stored ones, but
/// for a `scheme` or `bad_scheme` one, judged by the policy's schemes, and for
/// a response whose status the policy judges, on the host that answered,
/// else on every host (a 3xx is a hop and never judged). The crawl's reason
/// stays where the judged outcome is the stored one, and goes otherwise (D79).
// deficiency: KD23 The answer is judged here and nowhere else, so a query
// reading `targets.outcome` and `targets.reason` itself reads the crawl's:
// `report::digest::cause`'s `target_removed` (a target a remapped status
// breaks is `unexplained`), `report_sitemap`'s entries and `report_roots`'
// `has_sitemap` (`report::views`), and the digest's `target_outcome`
// (`report::digest::sections`). A comparison's guard (D81) reaches none of them.
pub(super) fn link_verdicts_with() -> String {
    let schemes: Vec<String> = WORKING_SCHEMES.iter().map(|s| format!("'{s}'")).collect();
    LINK_VERDICTS_WITH
        .replace("$SCHEMES", &schemes.join(", "))
        .replace("$ANSWERED_HOST", host!("coalesce(th.final_url, th.url)"))
        .replace("$DOUBLED_HOST", host!("d.url"))
}

// upstream: UF2 -- `pr` maps each run to the one whose policy judges it, since
// `policy_run` in a join condition aborts a DuckDB built with assertions.
const LINK_VERDICTS_WITH: &str = r"
CREATE OR REPLACE MACRO link_verdicts_with(policy_run := NULL) AS TABLE
WITH pr AS (
    SELECT run_id, coalesce(policy_run, getvariable('judged_by')::UBIGINT, run_id) AS prun FROM runs
), th AS (
    SELECT th.*, $ANSWERED_HOST AS answered_host FROM targets th
), tj AS (
    SELECT th.run_id, th.url_id, th.url, th.final_url,
        CASE
            WHEN th.reason IN ($reason(scheme), $reason(bad_scheme)) THEN
                CASE WHEN list_contains(coalesce(sch.schemes, [$SCHEMES]),
                        lower(regexp_extract(th.url, '^([A-Za-z][A-Za-z0-9+.-]*):', 1)))
                    THEN ['unchecked', $reason(scheme)] ELSE ['broken_url', $reason(bad_scheme)] END
            WHEN th.status NOT BETWEEN 300 AND 399 AND coalesce(hm.value, dm.value) IS NOT NULL
                THEN [coalesce(hm.value, dm.value),
                    CASE WHEN coalesce(hm.value, dm.value) = th.outcome THEN th.reason END]
            ELSE [th.outcome, th.reason]
        END AS answer
    FROM th
    LEFT JOIN pr ON pr.run_id = th.run_id
    LEFT JOIN (SELECT run_id, list(value) AS schemes FROM policy_in_force WHERE key = 'schemes'
        GROUP BY run_id) sch ON sch.run_id = pr.prun
    LEFT JOIN policy_in_force hm ON hm.run_id = pr.prun
        AND hm.key = 'status.' || th.status::VARCHAR AND hm.host = th.answered_host
    LEFT JOIN policy_in_force dm ON dm.run_id = pr.prun
        AND dm.key = 'status.' || th.status::VARCHAR AND dm.host IS NULL
), dsl AS (
    SELECT d.run_id, d.url_id, d.own, d.twin_id
    FROM doubled_slash d
    LEFT JOIN pr ON pr.run_id = d.run_id
    LEFT JOIN policy_in_force hp ON hp.run_id = pr.prun
        AND hp.key = 'doubled_slash' AND hp.host = $DOUBLED_HOST
    LEFT JOIN policy_in_force dp ON dp.run_id = pr.prun
        AND dp.key = 'doubled_slash' AND dp.host IS NULL
    WHERE coalesce(hp.value, dp.value, 'true') <> 'false'
), joined AS (
    SELECT l.run_id, l.src_url_id, l.target_url_id, l.fragment, l.raw_href, l.kind,
        l.xref_refspec,
        t.url_id IS NOT NULL AS found,
        t.answer[1] AS target_outcome,
        t.answer[2] AS target_reason,
        t.final_url IS NOT NULL AS redirected,
        coalesce(t.final_url <> t.url
            AND regexp_replace(t.final_url, '/([?].*)?$', '\1')
                = regexp_replace(t.url, '/([?].*)?$', '\1'), false) AS slash_only,
        p.url_id AS page_id,
        coalesce(try(url_decode(l.fragment)), l.fragment) AS decoded,
        d.url_id IS NOT NULL AS doubled,
        sd.url_id IS NOT NULL AS src_doubled,
        sd.twin_id IS NOT NULL AS src_twin,
        NOT regexp_matches(l.raw_href, '^([A-Za-z][A-Za-z0-9+.-]*:|[/#]|$)') AS relative
    FROM links l
    LEFT JOIN tj t ON t.run_id = l.run_id AND t.url_id = l.target_url_id
    LEFT JOIN dsl d ON d.run_id = t.run_id AND d.url_id = t.url_id
    LEFT JOIN dsl sd ON sd.run_id = l.run_id AND sd.url_id = l.src_url_id AND sd.own
    LEFT JOIN targets landing ON landing.run_id = t.run_id AND landing.url = t.final_url
    LEFT JOIN pages p ON p.run_id = t.run_id AND p.url_id = coalesce(landing.url_id, t.url_id)
), judged AS (
    SELECT joined.*, CASE
        WHEN xref_refspec IS NOT NULL AND (target_url_id = src_url_id OR fragment = xref_refspec
                OR target_outcome = 'broken_url')
            THEN ['broken_url', $reason(unresolved_xref)]
        WHEN NOT found THEN ['unchecked', $reason(cut)]
        WHEN doubled AND target_url_id <> src_url_id THEN ['broken_url', $reason(doubled_slash)]
        WHEN target_outcome <> 'ok' THEN [target_outcome, target_reason]
        WHEN coalesce(fragment, '') = '' OR lower(fragment) = 'top' THEN ['ok', NULL]
        WHEN contains(fragment, ':~:') THEN ['unchecked', $reason(text_fragment)]
        WHEN page_id IS NULL THEN ['unchecked', $reason(no_anchors)]
        WHEN EXISTS (SELECT 1 FROM anchors a WHERE a.run_id = joined.run_id
                AND a.url_id = joined.page_id AND a.anchor = joined.fragment)
            OR EXISTS (SELECT 1 FROM anchors a WHERE a.run_id = joined.run_id
                AND a.url_id = joined.page_id AND a.anchor = joined.decoded)
            THEN ['ok', NULL]
        ELSE ['broken_anchor', NULL]
    END AS verdict
    FROM joined
)
SELECT j.run_id, j.src_url_id, j.target_url_id, j.fragment, j.raw_href, j.kind, j.xref_refspec,
    j.verdict[1] AS outcome,
    CASE WHEN j.verdict[1] = 'ok' AND j.slash_only AND coalesce(pe.value, 'true') <> 'false'
            THEN $reason(pedantic_trailing_slash)
        ELSE j.verdict[2] END AS reason,
    j.redirected, j.relative,
    NOT j.src_doubled OR (NOT j.src_twin
        AND NOT (j.relative AND j.verdict[1] IN ('broken_url', 'broken_anchor', 'error'))) AS counted
FROM judged j
LEFT JOIN pr ON pr.run_id = j.run_id
LEFT JOIN policy_in_force pe ON pe.run_id = pr.prun AND pe.host IS NULL
    AND pe.key = 'pedantic_trailing_slash'";

/// A link and its verdict (D28), judged by its own run's policy unless D81's
/// guard names another (D77).
const LINK_VERDICTS: View = View {
    name: "link_verdicts",
    select: "SELECT * FROM link_verdicts_with()",
};

#[cfg(test)]
mod tests;
