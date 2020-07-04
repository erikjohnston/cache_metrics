use pyo3::prelude::*;
use pyo3::types::{PyDict};

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


    fn buckets(&self, py: Python) -> PyResult<Py<PyDict>> {
        let values = self.cache_metrics.stats().hits();

        let dict = PyDict::new(py);

        for (&percent, &count) in BUCKET_PERCENTAGES.iter().zip(values.iter()) {
            dict.set_item(percent, count)?;
        }

        // The last item in the returned list is the "inf" bucket.
        dict.set_item("+Inf", values.last())?;

        Ok(dict.into())
    }

    fn misses(&self) -> u128 {
        self.cache_metrics.stats().misses()
    }
}
