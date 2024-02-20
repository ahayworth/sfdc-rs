use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use sfdc::*;

fn criterion_benchmark(c: &mut Criterion) {
    let texts = vec![
        "Compression",
        "Absolute power corrupts absolutely",
        "The quick brown fox jumped over the lazy dog",
        "Those who cannot remember the past are condemned to repeat it",
    ];

    let mut group = c.benchmark_group("new");
    for text in &texts {
        let text = text.as_bytes();
        group.throughput(Throughput::Elements(text.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(text.len()), &text, |b, t| {
            b.iter(|| Sfdc::new(&t, 3));
        });
    }
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
