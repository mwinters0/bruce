//! Speed: the whole workload over the 30 docs.fp.o fixtures, the nightly
//! crawl's pages, one iteration being every page once. `finalize` is
//! included, being the same for all. The oracle is a tree-building parse,
//! shown for scale.

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use std::hint::black_box;
use tokenizer_spike::{finalize, fixtures, gum, gum_cb, lol, oracle, Raw};

fn bench(c: &mut Criterion) {
    let pages = fixtures(true);
    let bytes: usize = pages.iter().map(|(_, h)| h.len()).sum();
    let mut g = c.benchmark_group("docs-fp-o");
    g.throughput(Throughput::Bytes(bytes as u64));
    let runs: &[(&str, fn(&str) -> Raw)] = &[
        ("html5gum", gum::extract),
        ("html5gum-cb", gum_cb::extract),
        ("lol_html", lol::extract),
        ("oracle-html5ever", oracle::extract),
    ];
    for (name, f) in runs {
        g.bench_function(*name, |b| {
            b.iter(|| {
                for (_, html) in &pages {
                    black_box(finalize(f(black_box(html))));
                }
            })
        });
    }
    // Each tokenizer bare, counting start tags: what the workload adds on top.
    let bare: &[(&str, fn(&str) -> usize)] =
        &[("bare/html5gum", gum::count), ("bare/html5gum-cb", gum_cb::count), ("bare/lol_html", lol::count)];
    for (name, f) in bare {
        g.bench_function(*name, |b| {
            b.iter(|| {
                for (_, html) in &pages {
                    black_box(f(black_box(html)));
                }
            })
        });
    }
    g.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
