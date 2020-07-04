from typing import Any

from prometheus_client.core import REGISTRY, HistogramMetricFamily

from .cache_metrics import CacheMetrics


class _MetricsCollector:
    def __init__(self):
        self._caches = []

    def add_cache_metrics(self, name, cache_metrics):
        self._caches.append((name, cache_metrics))

    def collect(self):
        c = HistogramMetricFamily(
            "cache_metrics_hit_count_by_percentage_size",
            "Tracks cache hit count for percentage sizes of the cache",
            labels=("cache_name",),
        )

        for name, cache in list(self._caches):
            c.add_metric((name,), cache._cache.buckets(), None)

        yield c


_collector = _MetricsCollector()
REGISTRY.register(_collector)


class PrometheusCacheMetrics:
    """Reports cache metrics to prometheus
    """

    __slots__ = ["_cache"]

    def __init__(self, name: str, max_size: int):
        self._cache = CacheMetrics(max_size)

        _collector.add_cache_metrics(name, self)

    def insert(self, item: Any):
        self._cache.insert(item)
