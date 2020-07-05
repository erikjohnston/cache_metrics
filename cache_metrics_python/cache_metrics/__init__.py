from typing import Any

from prometheus_client.core import (REGISTRY, CounterMetricFamily,
                                    GaugeMetricFamily, HistogramMetricFamily)

from .cache_metrics import CacheMetrics


class _MetricsCollector:
    def __init__(self):
        self._caches = []

    def add_cache_metrics(self, name, cache_metrics):
        self._caches.append((name, cache_metrics))

    def collect(self):
        histo = HistogramMetricFamily(
            "cache_metrics_hit_count_by_percentage_size",
            "Tracks cache hit count for percentage sizes of the cache",
            labels=("cache_name",),
        )

        memory = GaugeMetricFamily(
            "cache_metrics_memory_usage",
            "Amount of memory each cache metric is currently using",
            labels=("cache_name",),
        )

        misses = CounterMetricFamily(
            "caches_metrics_misses",
            "Number of never before seen keys",
            labels=("cache_name",),
        )

        for name, cache in list(self._caches):
            histo.add_metric(
                (name,), [(str(k), v) for k, v in cache._cache.buckets()], None
            )
            memory.add_metric((name,), cache._cache.memory_usage())
            misses.add_metric((name,), cache._cache.misses())

        yield histo
        yield memory
        yield misses


_collector = _MetricsCollector()
REGISTRY.register(_collector)


class PrometheusCacheMetrics:
    """Reports cache metrics to prometheus
    """

    __slots__ = ["_cache"]

    def __init__(self, name: str, max_size: int):
        self._cache = CacheMetrics(max_size)

        _collector.add_cache_metrics(name, self)

    def change_cache_size(self, max_size):
        self._cache.max_size(max_size)

    def insert(self, item: Any):
        self._cache.insert(item)
