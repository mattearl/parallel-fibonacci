use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fibonacci_assesment::fibonacci;
use tokio::runtime::Builder;

fn criterion_benchmark(c: &mut Criterion) {
    let rt = Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Creating runtime failed");

    let size = black_box(100000);

    c.bench_function(format!("fib_seq_basic {size}").as_str(), |b| {
        b.iter(|| fibonacci::seq_basic(size))
    });

    c.bench_function(format!("fib_seq_hybrid_10000chunks {size}").as_str(), |b| {
        b.iter(|| fibonacci::seq_hybrid(size, 10000))
    });

    c.bench_function(
        format!("fib_seq_hybrid_kanal_1000chunks {size}").as_str(),
        |b| b.iter(|| fibonacci::seq_hybrid_kanal(size, 1000)),
    );

    c.bench_function(
        format!("fib_seq_hybrid_rayon_500chunks {size}").as_str(),
        |b| b.iter(|| fibonacci::seq_hybrid_rayon(size, 500)),
    );
    c.bench_function(
        format!("fib_seq_hybrid_rayon_1000chunks {size}").as_str(),
        |b| b.iter(|| fibonacci::seq_hybrid_rayon(size, 1000)),
    );
    c.bench_function(
        format!("fib_seq_hybrid_rayon_1500chunks {size}").as_str(),
        |b| b.iter(|| fibonacci::seq_hybrid_rayon(size, 1500)),
    );

    c.bench_function(format!("fib_hybrid_tokio_500chunks {size}").as_str(), |b| {
        b.to_async(&rt)
            .iter(|| fibonacci::seq_hybrid_tokio(size, 500, 20))
    });
    c.bench_function(
        format!("fib_hybrid_tokio_1000chunks {size}").as_str(),
        |b| {
            b.to_async(&rt)
                .iter(|| fibonacci::seq_hybrid_tokio(size, 1000, 20))
        },
    );
    c.bench_function(
        format!("fib_hybrid_tokio_1000chunks_30concurrent {size}").as_str(),
        |b| {
            b.to_async(&rt)
                .iter(|| fibonacci::seq_hybrid_tokio(size, 1000, 30))
        },
    );
    c.bench_function(
        format!("fib_hybrid_tokio_1500chunks {size}").as_str(),
        |b| {
            b.to_async(&rt)
                .iter(|| fibonacci::seq_hybrid_tokio(size, 1500, 20))
        },
    );

    c.bench_function(
        format!("fib_hybrid_kanal_tokio_700chunks {size}").as_str(),
        |b| {
            b.to_async(&rt)
                .iter(|| fibonacci::seq_hybrid_kanal_tokio(size, 700, 20))
        },
    );
    c.bench_function(
        format!("fib_hybrid_kanal_tokio_1000chunks {size}").as_str(),
        |b| {
            b.to_async(&rt)
                .iter(|| fibonacci::seq_hybrid_kanal_tokio(size, 1000, 20))
        },
    );
    c.bench_function(
        format!("fib_hybrid_kanal_tokio_1500chunks {size}").as_str(),
        |b| {
            b.to_async(&rt)
                .iter(|| fibonacci::seq_hybrid_kanal_tokio(size, 1500, 20))
        },
    );
    c.bench_function(
        format!("fib_hybrid_kanal_tokio_2000chunks {size}").as_str(),
        |b| {
            b.to_async(&rt)
                .iter(|| fibonacci::seq_hybrid_kanal_tokio(size, 2000, 20))
        },
    );
    c.bench_function(
        format!("fib_hybrid_kanal_tokio_1000chunks_30concurrent {size}").as_str(),
        |b| {
            b.to_async(&rt)
                .iter(|| fibonacci::seq_hybrid_kanal_tokio(size, 1000, 30))
        },
    );
    c.bench_function(
        format!("fib_hybrid_kanal_tokio_5000chunks {size}").as_str(),
        |b| {
            b.to_async(&rt)
                .iter(|| fibonacci::seq_hybrid_kanal_tokio(size, 5000, 20))
        },
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
