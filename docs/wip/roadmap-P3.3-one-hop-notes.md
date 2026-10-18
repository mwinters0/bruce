# P3.3 One hop over HTTP: notes

What P3.4 onward inherits, and what was refused.

- **The trait is `Fetcher::fetch(&self, &Url, Mode) -> Result<Response,
  Failure>`**, its future `Send`. `Mode::Page` is a GET whose 2xx HTML body is
  read; `Mode::Asset` is HEAD with the GET fallback and never reads a body.
  `Response` is `status`, the raw `location`, the raw `content_type` and a
  `Body`: `Unread`, `Html { text, size }` (`size` the decompressed bytes) or
  `TooLarge`. The crawler resolves `Location` and follows hops (P3.5); a 3xx
  is returned as the answer, in either mode.
- **A `Failure` is a `FailureKind` plus the error's words.** D16 maps
  `NxDomain` to `broken_url` and every other kind (`Timeout`, `Tls`,
  `Connect`, `Reset`, `Other`) to `error`; `as_str` is the `targets.reason`.
  That mapping is `crawl/outcome.rs`'s (P3.5). A status is never a `Failure`:
  a 5xx that outlasts its retries comes back as a `Response`.
- **Where P3.4 hooks in**: `HttpFetcher::get`'s loop is the only retry, and
  `once` the only place a request leaves. A per-host wait belongs before
  each `once`, and "stretched to the host's own interval" is the wait
  `retry_plan` and the backoff step already produce, taken as a floor.
  `HttpFetcher` is `Clone` and shares its pool; `Settings::user_agent` is
  where the top-level `user_agent` key lands.
- **Calls made beyond the spec**, none touching a contract:
  - A **429 with no `Retry-After` is retried on the backoff schedule**, as a
    5xx is (D8 says back off on 429). A 429 or 503 with a `Retry-After`
    within 60 s waits that long and spends one retry; one over it returns
    unretried. `Retry-After` on any other status is ignored. Both forms are
    read, seconds and an HTTP date (`httpdate`), a past date meaning no wait.
  - An **asset whose HEAD fails transiently** (timeout, lost connection) is
    asked again as a GET with the full retry loop; one whose HEAD fails for
    good (NXDOMAIN, TLS, refused) returns that failure without a second
    request.
  - A 2xx with **no `Content-Type` is not read**: "only a 2xx `text/html` or
    `application/xhtml+xml` body reaches the extractor".
  - **Compression is gzip and deflate**, not brotli: docs.fp.o answers
    gzip (the spec's "Facts found while grilling"), and advertising brotli
    would add a codec for no host we know. The Windows and macOS system proxy (`system-proxy`) is off; reqwest's
    environment-variable proxies still apply.
  - `hyper` is a direct dependency only to recognise a truncated response
    (`is_incomplete_message`); `httpdate` and `flate2` (test fixtures) are
    already in the tree.
- **The cap is applied while reading**, chunk by chunk after decompression,
  so a gzip bomb stops at 16 MiB. `Content-Length` is not consulted: it counts
  the compressed bytes.
- **The charset scan** (`fetch/charset.rs`) walks comments and every tag's
  attributes, takes the first `<meta>` naming a `charset` or a content-type
  pragma, and treats a UTF-16 label as UTF-8. It does not implement the
  WHATWG prescan's corners; none of the fixtures needs them.
- **Observed once, not committed as a test** (it needs the network): against
  the live resolver `*.invalid` and an unregistered `fedoraproject.org`
  subdomain both classify `NxDomain` although the resolver tried the
  machine's search domain first, and `expired.badssl.com` classifies `Tls`.
- **Negative result**: a TLS failure is not on `source()`'s path; it is
  inside the `io::Error` a handshake raises, so `cause` descends `get_ref`.
  A server that is plain HTTP, reached over `https://`, shows as `Tls` and is
  the test for it.
