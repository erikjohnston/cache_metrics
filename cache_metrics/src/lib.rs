use std::collections::hash_map::DefaultHasher;
use std::collections::VecDeque;
use std::{
    hash::{Hash, Hasher},
    time::{Duration, Instant},
};

use cuckoofilter::CuckooFilter;
use rand::rngs::OsRng;
use scalable_cuckoo_filter::{ScalableCuckooFilter, ScalableCuckooFilterBuilder};

pub const BUCKET_PERCENTAGES: [u16; 9] = [25, 50, 75, 90, 100, 110, 150, 200, 500];

pub const ALL_KEY_DURATION: Duration = Duration::from_secs(3 * 60 * 60);
pub const ALL_KEY_NUM_BUCKETS: usize = 12;

#[derive(Default, Debug)]
pub struct BucketStats {
    bucket_values: [u128; 10],
    misses: u128,
}

impl BucketStats {
    pub fn hit(&mut self, val: u16) {
        let pos = BUCKET_PERCENTAGES
            .iter()
            .position(|&x| val <= x)
            .unwrap_or_else(|| BUCKET_PERCENTAGES.len());
        self.bucket_values[pos] += 1;
    }

    pub fn hit_inf(&mut self) {
        self.bucket_values[BUCKET_PERCENTAGES.len()] += 1;
    }

    pub fn miss(&mut self) {
        self.misses += 1
    }

    pub fn hits(&self) -> &[u128; 10] {
        &self.bucket_values
    }

    pub fn misses(&self) -> u128 {
        self.misses
    }
}

pub struct Cache {
    queue: VecDeque<CuckooFilter<DefaultHasher>>,
    all_keys: Vec<ScalableCuckooFilter<u64, DefaultHasher, OsRng>>,
    max_size: u64,
    max_bucket_size: u64,
    stats: BucketStats,
    last_all_key_rotation: Instant,
}

impl Cache {
    pub fn new(max_size: u64) -> Cache {
        let all_keys = (0..ALL_KEY_NUM_BUCKETS)
            .into_iter()
            .map(|_| Cache::create_all_key_bucket())
            .collect();

        Cache {
            max_size,
            all_keys,
            queue: VecDeque::new(),
            max_bucket_size: max_size / 10,
            stats: BucketStats::default(),
            last_all_key_rotation: Instant::now(),
        }
    }

    fn create_all_key_bucket() -> ScalableCuckooFilter<u64, DefaultHasher, OsRng> {
        ScalableCuckooFilterBuilder::new()
            .false_positive_probability(0.01)
            .rng(OsRng::new().expect("os rng"))
            .hasher(DefaultHasher::new())
            .finish()
    }

    pub fn change_cache_size(&mut self, max_size: u64) {
        self.max_size = max_size;
        self.max_bucket_size = max_size / 10;
    }

    pub fn insert<T: Hash>(&mut self, item: T) {
        let mut hasher = DefaultHasher::new();
        item.hash(&mut hasher);
        let item_hash = hasher.finish();

        if let Some(pos) = self.queue.iter().position(|bucket| bucket.contains(&item)) {
            let length: u64 = self.queue.iter().take(pos).map(|bucket| bucket.len()).sum();

            let bottom = length;
            let top = length + self.queue[pos].len();

            let percentage = 100 * ((bottom + top) / 2) / self.max_size;
            self.stats.hit(percentage as u16);
        } else if self.all_keys.iter().any(|s| s.contains(&item_hash)) {
            self.stats.hit_inf();
        } else {
            self.stats.miss();
        }

        self.all_keys[0].insert(&item_hash);

        if Instant::now() - self.last_all_key_rotation > ALL_KEY_DURATION {
            self.all_keys.rotate_right(1);
            self.all_keys[0] = Cache::create_all_key_bucket();
            self.last_all_key_rotation = Instant::now();
        }

        if self.queue.is_empty() || self.queue[0].len() > self.max_bucket_size {
            self.queue.push_front(CuckooFilter::new())
        }

        for filter in &mut self.queue {
            filter.delete(&item);
        }

        self.queue[0].add(&item);

        let mut total_size = 0;
        if let Some(pos) = self.queue.iter().position(|x| {
            let start_size = total_size;
            total_size += x.len();
            start_size >= 5 * self.max_size
        }) {
            self.queue.truncate(pos)
        }
    }

    pub fn stats(&self) -> &BucketStats {
        &self.stats
    }

    pub fn memory_usage(&self) -> usize {
        let queue_mem: usize = self.queue.iter().map(|filter| filter.memory_usage()).sum();
        let all_keys_mem: usize = self.all_keys.iter().map(|s| s.bits() as usize / 8).sum();

        queue_mem + all_keys_mem
    }
}

#[cfg(test)]
mod tests {
    use super::{Cache, BUCKET_PERCENTAGES};

    #[test]
    fn simple_hit() {
        let mut cache = Cache::new(500);

        // First insert should miss
        cache.insert(5);
        assert_eq!(cache.stats().misses(), 1);

        // Second insert should hit
        cache.insert(5);
        assert_eq!(cache.stats().hits()[0], 1);
    }

    #[test]
    fn too_small() {
        let mut cache = Cache::new(1000);

        for i in 0..1300 {
            cache.insert(i);
        }

        assert_eq!(cache.stats().misses(), 1300);

        for i in 0..1300 {
            cache.insert(i);
        }

        // If cache was 1.5x the size we'd hit all the above.
        assert_eq!(cache.stats().hits()[6], 1300)
    }

    #[test]
    fn varied() {
        let mut cache = Cache::new(1000);

        for i in 0..1300 {
            cache.insert(i);
        }

        assert_eq!(cache.stats().misses(), 1300);

        for i in 0..1300 {
            cache.insert(i);
        }

        println!("{:?}", BUCKET_PERCENTAGES);
        println!("{:?}", cache.stats().bucket_values);

        assert_eq!(cache.stats().hits()[6], 1300)
    }
}
