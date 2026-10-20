//! The views the store ships, installed at every open beside the profiles'
//! (D6, D28).
//!
//! `link_verdicts` is a link's verdict, computed whenever it is read (D28): the
//! link's outcome, reason and whether its target was redirected, read off
//! `links`, its target, the target's landing target and the landing page's
//! `anchors`. A column holds the link's own fields, so a report reads one
//! relation. The first rule that applies decides:
//!
//! 1. A link with `xref_refspec` that points at its own page, or whose
//!    fragment is the refspec, is `broken_url`, reason `unresolved_xref`,
//!    whatever its target answered (D20). A CI run that resolved the refspec
//!    points the link at the page it names with the refspec's own fragment,
//!    which is neither, and keeps the refspec, and rules 2 to 7 judge it
//!    (D55).
//! 2. A link whose target has no row, which only a stopped run leaves, is
//!    `unchecked`, reason `cut`.
//! 3. A target that is not `ok` gives its link its own outcome and reason: the
//!    fragment is not looked at (D16).
//! 4. No fragment, an empty one, and `top` in any case are `ok` (D12). `top`
//!    is a browser's own and HTML matches it without regard to case.
//! 5. A text fragment (`:~:`) is `unchecked`, reason `text_fragment` (D12).
//! 6. An `ok` target with no page to read (`too_large`, an asset, an
//!    allowlisted URL first named by a non-anchor link) is `unchecked`,
//!    reason `no_anchors`: nothing is known, so nothing is broken.
//! 7. Otherwise the fragment is checked against the anchors of the page the
//!    target landed on (D46): `ok` where an anchor is the fragment as written
//!    or percent-decoded, as a browser looks (D12), else `broken_anchor`.

use crate::profile::View;

/// Every view the store ships.
pub(super) const ALL: &[View] = &[LINK_VERDICTS];

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
        p.url_id AS page_id,
        coalesce(try(url_decode(l.fragment)), l.fragment) AS decoded
    FROM links l
    LEFT JOIN targets t ON t.run_id = l.run_id AND t.url_id = l.target_url_id
    LEFT JOIN targets landing ON landing.run_id = t.run_id AND landing.url = t.final_url
    LEFT JOIN pages p ON p.run_id = t.run_id AND p.url_id = coalesce(landing.url_id, t.url_id)
), judged AS (
    SELECT joined.*, CASE
        WHEN xref_refspec IS NOT NULL AND (target_url_id = src_url_id OR fragment = xref_refspec)
            THEN ['broken_url', 'unresolved_xref']
        WHEN NOT found THEN ['unchecked', 'cut']
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
    verdict[1] AS outcome, verdict[2] AS reason, redirected
FROM judged",
};

#[cfg(test)]
mod tests;
