"""Domain service for analytics, health, and metrics."""

import asyncio

from contexter_server.core.bridge import StorageEngine
from contexter_server.models.analytics import (
    AnalyticsOverview,
    CostMetrics,
    PerformanceMetrics,
    ResourceUsage,
    ServiceStatus,
    SystemHealth,
)


def _safe_get(data: dict | Exception | None, key: str, default=0):
    """Safely extract a key from a dict, returning *default* if *data* is not a dict."""
    if isinstance(data, dict):
        return data.get(key, default)
    return default


class AnalyticsService:
    """Domain service for analytics and system monitoring."""

    def __init__(self, engine: StorageEngine) -> None:
        self._engine = engine

    async def get_overview(self) -> AnalyticsOverview:
        """Get a high-level overview of system analytics."""
        telemetry, storage, status = await asyncio.gather(
            self._engine.cache_telemetry(),
            self._engine.storage_size(),
            self._engine.status(),
            return_exceptions=True,
        )

        return AnalyticsOverview(
            total_sessions=_safe_get(telemetry, "total_sessions", 0),
            total_memories=_safe_get(telemetry, "total_memories", 0),
            total_agents=_safe_get(telemetry, "total_agents", 0),
            total_skills=_safe_get(telemetry, "total_skills", 0),
            storage_size_bytes=_safe_get(storage, "total_bytes", 0),
            uptime_seconds=_safe_get(status, "uptime_seconds", 0),
        )

    async def get_health(self) -> SystemHealth:
        """Get system health status."""
        status, telemetry, storage = await asyncio.gather(
            self._engine.status(),
            self._engine.cache_telemetry(),
            self._engine.storage_size(),
            return_exceptions=True,
        )

        return SystemHealth(
            status=_safe_get(status, "status", "ok"),
            uptime_seconds=_safe_get(status, "uptime_seconds", 0),
            memory_usage_mb=_safe_get(status, "memory_usage_mb", 0.0),
            storage_size_bytes=_safe_get(storage, "total_bytes", 0),
            cache_entries=_safe_get(telemetry, "cache_entries", 0),
        )

    async def get_performance(self) -> PerformanceMetrics:
        """Get performance metrics from bridge telemetry."""
        telemetry = await self._engine.cache_telemetry()

        return PerformanceMetrics(
            avg_response_time_ms=telemetry.get("avg_response_time_ms", 0.0),
            total_operations=telemetry.get("total_operations", 0),
            cache_hit_rate=telemetry.get("cache_hit_rate", 0.0),
        )

    async def get_resources(self) -> ResourceUsage:
        """Get current resource usage."""
        storage, status, telemetry = await asyncio.gather(
            self._engine.storage_size(),
            self._engine.status(),
            self._engine.cache_telemetry(),
            return_exceptions=True,
        )

        total_bytes = _safe_get(storage, "total_bytes", 0)

        return ResourceUsage(
            cpu_percent=_safe_get(status, "cpu_percent", 0.0),
            memory_mb=_safe_get(status, "memory_usage_mb", 0.0),
            storage_mb=total_bytes / (1024 * 1024),
        )

    async def get_costs(self) -> CostMetrics:
        """Get cost metrics (defaults — no cost tracking in bridge yet)."""
        return CostMetrics()

    async def get_service_status(self) -> ServiceStatus:
        """Get overall service status from bridge status."""
        status = await self._engine.status()
        return ServiceStatus(
            name="contexter-server",
            status=status.get("status", "ok"),
            latency_ms=status.get("latency_ms", 0.0),
        )
