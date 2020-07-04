use pyo3::prelude::*;
use pyo3::types::PyList;

use cache_metrics::{Cache, BUCKET_PERCENTAGES};

#[pymodule]
fn cache_metrics(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<CacheMetrics>()?;
    Ok(())
}

#[pyclass]
struct CacheMetrics {
    cache_metrics: Cache,
}

#[pymethods]
impl CacheMetrics {
    #[new]
    fn new(max_size: u64) -> Self {
        let cache_metrics = Cache::new(max_size);
        CacheMetrics { cache_metrics }
    }

    fn insert(&mut self, item: &PyAny) -> PyResult<()> {
        let hash = item.hash()?;

        self.cache_metrics.insert(hash);

        Ok(())
    }

    fn buckets(&self, py: Python) -> PyResult<Py<PyList>> {
        let values = self.cache_metrics.stats().hits();

        let list = PyList::empty(py);

        let mut cumalitive = 0;
        for (&percent, &count) in BUCKET_PERCENTAGES.iter().zip(values.iter()) {
            cumalitive += count;
            list.append((percent, cumalitive))?;
        }

        // The last item in the returned list is the "inf" bucket.
        list.append((
            "+Inf",
            cumalitive + values.last().expect("slice is non-empty"),
        ))?;

        Ok(list.into())
    }

    fn misses(&self) -> u128 {
        self.cache_metrics.stats().misses()
    }
}
