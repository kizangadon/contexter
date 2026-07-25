"""Tests for analytics Pydantic models."""

import pytest
from pydantic import ValidationError

from contexter_server.models.analytics import (
    AnalyticsOverview,
    SystemHealth,
    PerformanceMetrics,
    ResourceUsage,
    CostMetrics,
    ModelCost,
    ServiceStatus,
)


class TestAnalyticsModels:
    """Analytics model validation tests."""

    def test_analytics_overview_defaults(self):
        """AnalyticsOverview should have zero defaults."""
        overview = AnalyticsOverview()
        assert overview.total_sessions == 0
        assert overview.total_memories == 0
        assert overview.total_agents == 0
        assert overview.total_skills == 0
        assert overview.storage_size_bytes == 0
        assert overview.uptime_seconds == 0

    def test_analytics_overview_with_values(self):
        """AnalyticsOverview with values."""
        overview = AnalyticsOverview(
            total_sessions=10,
            total_memories=100,
            total_agents=3,
            total_skills=5,
            storage_size_bytes=4096,
            uptime_seconds=86400,
        )
        assert overview.total_sessions == 10
        assert overview.total_memories == 100

    def test_system_health_defaults(self):
        """SystemHealth should default to ok status."""
        health = SystemHealth()
        assert health.status == "ok"
        assert health.uptime_seconds == 0
        assert health.memory_usage_mb == 0.0
        assert health.storage_size_bytes == 0
        assert health.cache_entries == 0

    def test_system_health_custom(self):
        """SystemHealth with custom values."""
        health = SystemHealth(
            status="degraded",
            uptime_seconds=3600,
            memory_usage_mb=1024.5,
            storage_size_bytes=1000000,
            cache_entries=500,
        )
        assert health.status == "degraded"
        assert health.memory_usage_mb == 1024.5

    def test_performance_metrics_defaults(self):
        """PerformanceMetrics should have zero defaults."""
        perf = PerformanceMetrics()
        assert perf.avg_response_time_ms == 0.0
        assert perf.total_operations == 0
        assert perf.cache_hit_rate == 0.0

    def test_resource_usage_defaults(self):
        """ResourceUsage should have zero defaults."""
        r = ResourceUsage()
        assert r.cpu_percent == 0.0
        assert r.memory_mb == 0.0
        assert r.storage_mb == 0.0

    def test_cost_metrics_defaults(self):
        """CostMetrics should have empty cost_by_model."""
        c = CostMetrics()
        assert c.total_cost == 0.0
        assert c.cost_by_model == {}

    def test_cost_metrics_with_values(self):
        """CostMetrics with model costs."""
        c = CostMetrics(
            total_cost=10.5,
            cost_by_model={"gpt-4": 8.0, "claude-3": 2.5},
        )
        assert c.total_cost == 10.5
        assert c.cost_by_model["gpt-4"] == 8.0

    def test_model_cost_defaults(self):
        """ModelCost should have zero defaults."""
        m = ModelCost(model="gpt-4")
        assert m.model == "gpt-4"
        assert m.cost == 0.0
        assert m.tokens == 0
        assert m.operations == 0

    def test_service_status_defaults(self):
        """ServiceStatus should default status to ok."""
        s = ServiceStatus(name="engine")
        assert s.name == "engine"
        assert s.status == "ok"
        assert s.latency_ms == 0.0
