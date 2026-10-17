//! The parser bench (D24): the extraction pass over the 30 docs.fp.o
//! fixtures, the nightly crawl's pages, one iteration being every page once.
//! `mise run bench` times it; `mise run check` runs it once, in criterion's
//! test mode, to keep it building and running.

#[path = "../tests/common/mod.rs"]
mod common;

use std::hint::black_box;

use bruce::cli::registry;
use bruce::config::ProfileChoice;
use bruce::extract::extract;
use criterion::{Criterion, Throughput, criterion_group, criterion_main};

fn bench(c: &mut Criterion) {
    let pages: Vec<_> = common::fixtures()
        .into_iter()
        .filter(|f| f.path.starts_with("docs-fp-o/"))
        .collect();
    let registry = registry();
    let mut group = c.benchmark_group("docs-fp-o");
    group.throughput(Throughput::Bytes(
        pages.iter().map(|p| p.html.len() as u64).sum(),
    ));
    group.bench_function("extract", |b| {
        b.iter(|| {
            for page in &pages {
                black_box(
                    extract(
                        black_box(&page.html),
                        &page.url,
                        &ProfileChoice::Auto,
                        &registry,
                    )
                    .unwrap(),
                );
            }
        })
    });
    group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
