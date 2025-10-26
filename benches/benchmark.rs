use criterion::async_executor::FuturesExecutor;
use criterion::{Criterion, criterion_group, criterion_main};
use hash_finder::{async_impl, multithread_impl, sync_impl};

const N: u8 = 2;
const F: u32 = 1;
const T: u8 = 5;

fn criterion_benchmark(c: &mut Criterion) {
    let hash_finder_sync = sync_impl::HashFinder::new(N, F);
    let hash_finder_multithread = multithread_impl::HashFinder::new(T, N, F);
    let hash_finder_async = async_impl::HashFinder::new(T, N, F);

    let mut group = c.benchmark_group("hash_finder");

    group.bench_function("sync", |b| b.iter(|| hash_finder_sync.run()));

    group.bench_function("multithread", |b| b.iter(|| hash_finder_multithread.run()));

    group.bench_function("my_async_function", |b| {
        // Use AsyncBencher to run the async function
        b.to_async(FuturesExecutor)
            .iter(|| async { hash_finder_async.run().await });
    });

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
