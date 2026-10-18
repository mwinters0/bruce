# P3.7.2 An abandoned run holds whole targets: notes

What P3.8 onward inherits, and what was refused.

- **`crawl::Writes`**, from `Crawl::writes`, is a gate every write passes on
  the blocking pool: a target's (`write_blocking`) and the roots'
  `sitemap_file` links (`discover`). `close` refuses any write not yet begun
  and blocks until those begun end (D49). Only the second Ctrl-C closes it,
  after `shutdown_background` and before `Writer::finish`; the first stop
  never needs it, `drive` waiting for its tasks.
- **A write refused at the gate returns no targets and sends nothing**, so
  the target it carried is cut like one dropped mid-hop, though no summary
  is printed to count it.
- **The page's extraction and a sitemap's parse are not waited for**: their
  results reach no write once the crawl's future is dropped.
- **Negative result**: with `writes.close()` removed, the test
  `an_abandoned_run_holds_the_page_it_was_writing_whole` (20 000 links, a
  writer channel of one row) found 0 of the page's links written on three
  runs out of three, so the hazard is not rare.
- **A third Ctrl-C does nothing**, read from the code and not observed:
  tokio's SIGINT handler stays registered (`signal-hook-registry` never
  removes one) while the runtime that would deliver it is gone.
