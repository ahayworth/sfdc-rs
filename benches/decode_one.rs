use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use sfdc::*;

fn criterion_benchmark(c: &mut Criterion) {
    let texts = vec![
        "Compression",
        "Absolute power corrupts absolutely",
        "The quick brown fox jumped over the lazy dog",
        "Those who cannot remember the past are condemned to repeat it",
    ];

    let inputs = texts
        .iter()
        .map(|t| (t.as_bytes(), Sfdc::new(t.as_bytes(), 3)))
        .collect::<Vec<_>>();

    let mut group = c.benchmark_group("decode one");
    for (t, sfdc) in &inputs {
        group.throughput(Throughput::Elements(t.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(t.len()), &sfdc, |b, s| {
            b.iter(|| s.decode_one(2));
        });
    }
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
