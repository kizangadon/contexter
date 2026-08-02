"""Domain service for analytics, health, and metrics.

Telemetry translation boundary: the Rust engine emits distinct shapes per
call — ``cache_telemetry()`` is snake_case (``entries_by_type``,
``total_ops``), ``storage_size()`` is camelCase (``total``, ``perCf``,
``walSize``), and ``status()`` nests ``cacheTelemetry``. This service is the
anti-corruption layer that maps those engine shapes onto the analytics domain
models; every read is explicit and key mismatches are logged, never silently
defaulted (REQ-AN-003).
"""

import asyncio

import structlog

from contexter_server.core.bridge import StorageEngine
from contexter_server.models.analytics import (
    AnalyticsOverview,
    CostMetrics,
    PerformanceMetrics,
    ResourceUsage,
    ServiceStatus,
    SystemHealth,
)

logger = structlog.get_logger(__name__)

# NOTE: agent/skill counts use the bridge's dedicated store-backed counters
# (count_agents/count_skills). The engine exposes them with the same
# "Bypass — always reads from L2" policy as count_sessions/count_memories,
# avoiding the O(store) list_* full scans (REQ-ACE-003).


def _safe_get(data: dict | Exception | None, key: str, default=0):
    """Extract *key* from *data*, logging explicitly instead of silently defaulting.

    A dict payload missing the key is a mapping-drift signal -> debug.
    A non-dict payload (engine exception / empty response) is an operational
    condition -> debug. Either way the caller sees the default plus a log.
    """
    if isinstance(data, dict):
        if key in data:
            return data[key]
        logger.debug(
            "analytics.missing_key",
            key=key,
            payload_keys=sorted(data.keys()),
            default=default,
        )
        return default
    logger.debug(
        "analytics.non_dict_payload",
        key=key,
        payload_type=type(data).__name__,
    )
    return default


def _safe_int(data, key: str, default: int = 0) -> int:
    """Coerce an engine count to an int; log and default when it is not."""
    if isinstance(data, int) and not isinstance(data, bool):
        return data
    logger.debug(
        "analytics.non_integer_count",
        key=key,
        value_type=type(data).__name__,
        default=default,
    )
    return default


def _safe_cache_entries(telemetry, key: str = "entries_by_type") -> int:
    """Sum the engine's cache-resident entity counts (``entries_by_type``)."""
    entries = _safe_get(telemetry, key, {})
    if isinstance(entries, dict):
        return sum(entries.values())
    logger.debug(
        "analytics.invalid_entries_by_type",
        key=key,
        value_type=type(entries).__name__,
    )
    return 0


class AnalyticsService:
    """Domain service for analytics and system monitoring."""

    def __init__(self, engine: StorageEngine) -> None:
        self._engine = engine

    async def get_overview(self) -> AnalyticsOverview:
        """Get a high-level overview of system analytics.

        Counts come from the engine store (REQ-AN-001): sessions, memories,
        agents, and skills all via the bridge's dedicated store-backed
        counters (REQ-ACE-003 — no full-store list scans). Storage and
        uptime map from ``storage_size()``/``status()`` telemetry.
        """
        storage, status_, sessions, memories, agents, skills = (
            await asyncio.gather(
                self._engine.storage_size(),
                self._engine.status(),
                self._engine.count_sessions({}),
                self._engine.count_memories({}),
                self._engine.count_agents({}),
                self._engine.count_skills({}),
                return_exceptions=True,
            )
        )

        return AnalyticsOverview(
            total_sessions=_safe_int(sessions, "total_sessions"),
            total_memories=_safe_int(memories, "total_memories"),
            total_agents=_safe_int(agents, "total_agents"),
            total_skills=_safe_int(skills, "total_skills"),
            storage_size_bytes=_safe_get(storage, "total", 0),
            uptime_seconds=_safe_get(status_, "uptime_seconds", 0),
        )

    async def get_health(self) -> SystemHealth:
        """Get system health status mapped from real engine telemetry (REQ-AN-002)."""
        status_, telemetry, storage = await asyncio.gather(
            self._engine.status(),
            self._engine.cache_telemetry(),
            self._engine.storage_size(),
            return_exceptions=True,
        )

        return SystemHealth(
            status=_safe_get(status_, "status", "ok"),
            uptime_seconds=_safe_get(status_, "uptime_seconds", 0),
            memory_usage_mb=_safe_get(status_, "memory_usage_mb", 0.0),
            storage_size_bytes=_safe_get(storage, "total", 0),
            cache_entries=_safe_cache_entries(telemetry),
        )

    async def get_performance(self) -> PerformanceMetrics:
        """Get performance metrics from bridge telemetry.

        The engine exposes ``total_ops`` (snake_case) for total operations and
        ``hits``/``misses`` for a computed cache hit rate.
        """
        telemetry = await self._engine.cache_telemetry()

        hits = _safe_get(telemetry, "hits", 0)
        misses = _safe_get(telemetry, "misses", 0)
        attempts = hits + misses
        cache_hit_rate = hits / attempts if attempts > 0 else 0.0

        return PerformanceMetrics(
            avg_response_time_ms=_safe_get(telemetry, "avg_response_time_ms", 0.0),
            total_operations=_safe_get(telemetry, "total_ops", 0),
            cache_hit_rate=cache_hit_rate,
        )

    async def get_resources(self) -> ResourceUsage:
        """Get current resource usage.

        Storage derives from the engine's camelCase ``total``; the engine
        exposes no CPU/memory telemetry, so those remain graceful defaults.
        """
        storage, status_, telemetry = await asyncio.gather(
            self._engine.storage_size(),
            self._engine.status(),
            self._engine.cache_telemetry(),
            return_exceptions=True,
        )

        total_bytes = _safe_get(storage, "total", 0)

        return ResourceUsage(
            cpu_percent=_safe_get(status_, "cpu_percent", 0.0),
            memory_mb=_safe_get(status_, "memory_usage_mb", 0.0),
            storage_mb=total_bytes / (1024 * 1024),
        )

    async def get_costs(self) -> CostMetrics:
        """Get cost metrics (defaults — no cost tracking in bridge yet)."""
        return CostMetrics()

    async def get_service_status(self) -> ServiceStatus:
        """Get overall service status from bridge status."""
        status_ = await self._engine.status()
        return ServiceStatus(
            name="contexter-server",
            status=_safe_get(status_, "status", "ok"),
            latency_ms=_safe_get(status_, "latency_ms", 0.0),
        )
