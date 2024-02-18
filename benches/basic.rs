use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use sfdc::*;

fn criterion_benchmark(c: &mut Criterion) {
    let texts = vec![
        "Compression",
        "Absolute power corrupts absolutely",
        "The quick brown fox jumped over the lazy dog",
        "Those who cannot remember the past are condemned to repeat it",
    ];

    let texts = texts
        .iter()
        .map(|t| {
            let t = t.split("").collect::<Vec<_>>();
            t[1..t.len() - 1].to_vec()
        })
        .collect::<Vec<_>>();

    let mut group = c.benchmark_group("new");
    for text in &texts {
        group.throughput(Throughput::Elements(text.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(text.len()), &text, |b, t| {
            b.iter(|| Sfdc::new(&t, 3));
        });
    }
    group.finish();

    let inputs = texts
        .iter()
        .map(|t| (t, Sfdc::new(&t, 3)))
        .collect::<Vec<_>>();

    let mut group = c.benchmark_group("decode all");
    for (t, sfdc) in &inputs {
        group.throughput(Throughput::Elements(t.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(t.len()), &sfdc, |b, s| {
            b.iter(|| s.decode_range(0, t.len()));
        });
    }
    group.finish();

    let mut group = c.benchmark_group("decode range");
    for (t, sfdc) in &inputs {
        group.throughput(Throughput::Elements(t.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(t.len()), &sfdc, |b, s| {
            b.iter(|| s.decode_range(1, 3));
        });
    }
    group.finish();

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
