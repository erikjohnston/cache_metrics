#![feature(test)]

extern crate test;

use cache_metrics::Cache;

use test::{Bencher};

#[bench]
fn bench_hit(b: &mut Bencher) {
    let mut cache = Cache::new(500);

    b.iter(|| {
        cache.insert(&5);
    });
}


#[bench]
fn bench_miss(b: &mut Bencher) {
    let mut cache = Cache::new(1000000);

    // Pre fill the cache with garbage
    for i in 0..2000000 {
        cache.insert(&-i);
    }

    let mut i = 0;

    b.iter(|| {
        i += 1;
        cache.insert(&i);
    });
}
