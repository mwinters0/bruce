# Handing off a long job

Read this before launching a job you expect to run **more than 30 minutes** — a
full `bruce crawl` of docs.fedoraproject.org at its `Crawl-delay`, anything at
that scale. `CLAUDE.md`'s
protocol still binds: detached, `setsid`, stdout and stderr to a file under
`runs/`, never waited on. Three obligations on top, because the agent that reads
the results is a fresh one, hours from now, with none of your context.

## 1. Start it, then watch for five minutes

Confirm the job is *making progress*, not merely alive: sample twice, a few
minutes apart, checking something that must move — the log's byte count, the
output file's size, a row or block counter. Five minutes is a ceiling, not a
target; stop early once progress is evident, and if it is not, kill the job by
process group, fix it, and start over. Never hand off a job you have not seen
working.

## 2. Write the handoff doc

`runs/<job-name>-<YYYYMMDD-HHMM>/HANDOFF.md`, beside the log — a gitignored
`runs/` artifact consumed by nothing in the repo. Everything load-bearing goes in
the **frontmatter**, the reader being a cron fire that will read only that; below
it, write whatever prose the follow-up agent needs — what the job is proving,
which counts matter, what an unexpected result would mean.

```yaml
---
job: crawl-docs-fp-o
slice: 4.3
started: 2026-11-02T14:07:00-05:00     # absolute, with offset
expect: ~45m
log: runs/crawl-20261102-1407/log
check: test -f runs/crawl-20261102-1407/exit && echo exited || (kill -0 41234 2>/dev/null && echo running || echo vanished)
progress: wc -c < runs/crawl-20261102-1407/log
exit: cat runs/crawl-20261102-1407/exit
running: check says running
done: check says exited; then exit 0 = ok, 1 = broken links found (still ok), 130 = SIGINT, 143 = SIGTERM
failed: exit not 0/1; or check says vanished; or log tail has "panicked"/"No space left"
stuck: progress unchanged across two consecutive fires
stop: pkill -g 41234     # the run so far stays in the DB, marked unfinished
volatile: runs/crawl-20261102-1407/crawl.duckdb (a re-run with the same --db appends a new run_id)
result: runs/crawl-20261102-1407/crawl.duckdb, with its run_id in the log's last line
next: run `bruce report stats` on the result, check the slice's acceptance row, tick 4.3, notes doc
---
```

- `check`, `progress`, `exit` and `stop` are **single commands runnable verbatim
  from the repo root** — no placeholders, no variables the reader lacks, no
  "adjust the path"; they will be pasted, not interpreted. `progress` must print
  a number that only ever goes up.
- `running`, `done`, `failed` and `stuck` say how to read that output in as few
  words as carry the meaning, and **name every exit code the job can plausibly
  produce**; silence about one is the reader guessing.
- **A bare process records its own exit status**: the shell `setsid` starts
  writes it once the job ends, so that shell must outlive the job —
  `setsid bash -c 'cmd > log 2>&1; echo $? > exit' &`. Never `exec` the job
  inside it: `exec` replaces the shell, the `echo` never runs, and `check`
  can only ever say the job vanished, even of one that finished cleanly.
- `stop` ends the job safely and says what that costs — for a bare process, a
  process-group kill (`pkill -g <pgid>`) with the pgid written out. `started` and
  `expect` let the reader compute elapsed time without asking.
- `volatile` names every piece of state **gone or overwritten** by the time
  someone investigates: tmpfs staging, container state a prune destroys, scratch
  the next run reuses, temp files. A diagnosis agent can look only where this
  line points.
- `next` is one line: what the follow-up agent does with the result. Not how.

## 3. Report the path and stop

Short: the handoff doc's path, the slice it belongs to, and one line on what you
saw in the five minutes. **Do not tick the slice's box** — annotate the checklist
entry with what remains and why it waits. The orchestrator may come back once to
have you fix a frontmatter line it could not run, and nothing else.
