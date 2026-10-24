//! The views the store ships, installed at every open beside the profiles'
//! (D6, D28).
//!
//! `link_verdicts` is a link's verdict, computed whenever it is read (D28): the
//! link's outcome, reason and whether its target was redirected, read off
//! `links`, its target, the target's landing target and the landing page's
//! `anchors`. A column holds the link's own fields, so a report reads one
//! relation. The first rule that applies decides:
//!
//! 1. A link with `xref_refspec` that points at its own page, whose
//!    fragment is the refspec, or whose target is `broken_url`, is
//!    `broken_url`, reason `unresolved_xref`, whatever its target answered
//!    (D20). A CI run that resolved the refspec points the link at the page it
//!    names with the refspec's own fragment, and keeps the refspec: rules 2 to
//!    7 judge it (D55), unless the URL it names, which Resolve may only have
//!    derived, is itself broken (D57).
//! 2. A link whose target has no row, which only a stopped run leaves, is
//!    `unchecked`, reason `cut`.
//! 3. A link to a URL whose path doubles a slash, or to one that redirected
//!    to such a URL, is `broken_url`, reason [`DOUBLED_SLASH`], whatever the
//!    target answered and asked or not, bar a page's link to itself, a
//!    `#fragment` link on a doubled-slash page being that page's (D71).
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
//! added or removed has the reason [`PEDANTIC_TRAILING_SLASH`], whatever the
//! hops between (D70). It is not an outcome of its own.
//!
//! `relative` says the link's `raw_href` keeps its page's path: it is resolved
//! against it (`../`, `committee/`) or is a `?query` on it. `counted` says the
//! link counts in the reports (D71): not a link on a page whose URL doubles a
//! slash and whose twin, the URL with its slashes collapsed, is a page of the
//! run, and not a broken `relative` one on such a page without a twin, which
//! the doubled slash breaks and the linking page answers for. `doubled_slash`
//! lists the targets whose URL doubles a slash, or whose landing does, and for
//! the first the page of the run at its twin's URL, `twin_id`.

use crate::profile::View;

/// The reason of an `ok` link whose redirect differs from its URL only by a
/// final `/` (D70).
pub const PEDANTIC_TRAILING_SLASH: &str = "pedantic_trailing_slash";

/// The reason of a link to a URL whose path doubles a slash (D71).
pub const DOUBLED_SLASH: &str = "doubled_slash";

/// Every view the store ships, `doubled_slash` before the view that joins it.
pub(super) const ALL: &[View] = &[DOUBLED_SLASH_TARGETS, LINK_VERDICTS];

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

/// The targets whose URL doubles a slash (`own`) or that redirected to a URL
/// that does, and for the first its `twin_id`, the page of the run at its
/// URL with the slashes collapsed, if any (D71). It joins the few, not
/// every target of the run, to look for the twin.
const DOUBLED_SLASH_TARGETS: View = View {
    name: "doubled_slash",
    select: concat!(
        "\
SELECT d.run_id, d.url_id, d.own, min(tp.url_id) AS twin_id
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
GROUP BY d.run_id, d.url_id, d.own"
    ),
};

/// A link and its verdict (D28). The landing target is the one `final_url`
/// names, and a target that was not redirected is its own landing page.
const LINK_VERDICTS: View = View {
    name: "link_verdicts",
    select: "\
WITH joined AS (
    SELECT l.run_id, l.src_url_id, l.target_url_id, l.fragment, l.raw_href, l.kind,
        l.xref_refspec,
        t.url_id IS NOT NULL AS found,
        t.outcome AS target_outcome,
        t.reason AS target_reason,
        t.final_url IS NOT NULL AS redirected,
        coalesce(t.final_url <> t.url
            AND regexp_replace(t.final_url, '/([?].*)?$', '\\1')
                = regexp_replace(t.url, '/([?].*)?$', '\\1'), false) AS slash_only,
        p.url_id AS page_id,
        coalesce(try(url_decode(l.fragment)), l.fragment) AS decoded,
        d.url_id IS NOT NULL AS doubled,
        sd.url_id IS NOT NULL AS src_doubled,
        sd.twin_id IS NOT NULL AS src_twin,
        NOT regexp_matches(l.raw_href, '^([A-Za-z][A-Za-z0-9+.-]*:|[/#]|$)') AS relative
    FROM links l
    LEFT JOIN targets t ON t.run_id = l.run_id AND t.url_id = l.target_url_id
    LEFT JOIN doubled_slash d ON d.run_id = t.run_id AND d.url_id = t.url_id
    LEFT JOIN doubled_slash sd ON sd.run_id = l.run_id AND sd.url_id = l.src_url_id AND sd.own
    LEFT JOIN targets landing ON landing.run_id = t.run_id AND landing.url = t.final_url
    LEFT JOIN pages p ON p.run_id = t.run_id AND p.url_id = coalesce(landing.url_id, t.url_id)
), judged AS (
    SELECT joined.*, CASE
        WHEN xref_refspec IS NOT NULL AND (target_url_id = src_url_id OR fragment = xref_refspec
                OR target_outcome = 'broken_url')
            THEN ['broken_url', 'unresolved_xref']
        WHEN NOT found THEN ['unchecked', 'cut']
        WHEN doubled AND target_url_id <> src_url_id THEN ['broken_url', 'doubled_slash']
        WHEN target_outcome <> 'ok' THEN [target_outcome, target_reason]
        WHEN coalesce(fragment, '') = '' OR lower(fragment) = 'top' THEN ['ok', NULL]
        WHEN contains(fragment, ':~:') THEN ['unchecked', 'text_fragment']
        WHEN page_id IS NULL THEN ['unchecked', 'no_anchors']
        WHEN EXISTS (SELECT 1 FROM anchors a WHERE a.run_id = joined.run_id
                AND a.url_id = joined.page_id AND a.anchor = joined.fragment)
            OR EXISTS (SELECT 1 FROM anchors a WHERE a.run_id = joined.run_id
                AND a.url_id = joined.page_id AND a.anchor = joined.decoded)
            THEN ['ok', NULL]
        ELSE ['broken_anchor', NULL]
    END AS verdict
    FROM joined
)
SELECT run_id, src_url_id, target_url_id, fragment, raw_href, kind, xref_refspec,
    verdict[1] AS outcome,
    CASE WHEN verdict[1] = 'ok' AND slash_only THEN 'pedantic_trailing_slash'
        ELSE verdict[2] END AS reason,
    redirected, relative,
    NOT src_doubled OR (NOT src_twin
        AND NOT (relative AND verdict[1] IN ('broken_url', 'broken_anchor', 'error'))) AS counted
FROM judged",
};

#[cfg(test)]
mod tests;
