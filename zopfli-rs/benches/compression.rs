use criterion::{criterion_group, criterion_main, Criterion};
use zopfli::{ZopfliOptions, OutputType};

fn bench_compress(c: &mut Criterion) {
    let data = b"The quick brown fox jumps over the lazy dog. The quick brown fox jumps over the lazy dog.";
    
    c.bench_function("compress_c_gzip", |b| {
        b.iter(|| {
            let options = ZopfliOptions::default();
            zopfli::compress(&options, OutputType::Gzip, data)
        })
    });
}

criterion_group!(benches, bench_compress);
criterion_main!(benches);