"""Tests for AnalyticsService."""

from unittest.mock import AsyncMock

import pytest

from contexter_server.models.analytics import (
    AnalyticsOverview,
    CostMetrics,
    PerformanceMetrics,
    ResourceUsage,
    ServiceStatus,
    SystemHealth,
)
from contexter_server.services.analytics_service import AnalyticsService


@pytest.fixture
def mock_engine():
    engine = AsyncMock()
    return engine


@pytest.fixture
def service(mock_engine):
    return AnalyticsService(mock_engine)


class TestAnalyticsServiceGetOverview:
    """Tests for AnalyticsService.get_overview."""

    @pytest.mark.asyncio
    async def test_returns_overview(self, service, mock_engine):
        mock_engine.cache_telemetry.return_value = {
            "total_sessions": 10,
            "total_memories": 100,
            "total_agents": 3,
            "total_skills": 5,
        }
        mock_engine.storage_size.return_value = {"total_bytes": 1024000}
        mock_engine.status.return_value = {"uptime_seconds": 3600}

        result = await service.get_overview()
        assert isinstance(result, AnalyticsOverview)
        assert result.total_sessions == 10
        assert result.total_memories == 100
        assert result.total_agents == 3
        assert result.total_skills == 5
        assert result.storage_size_bytes == 1024000
        assert result.uptime_seconds == 3600

    @pytest.mark.asyncio
    async def test_returns_defaults_on_empty_telemetry(self, service, mock_engine):
        mock_engine.cache_telemetry.return_value = {}
        mock_engine.storage_size.return_value = {}
        mock_engine.status.return_value = {}

        result = await service.get_overview()
        assert result.total_sessions == 0
        assert result.storage_size_bytes == 0

    @pytest.mark.asyncio
    async def test_gathers_independent_calls(self, service, mock_engine):
        """Verify cache_telemetry, storage_size, and status are called concurrently."""
        mock_engine.cache_telemetry.return_value = {
            "total_sessions": 5,
            "total_memories": 50,
            "total_agents": 2,
            "total_skills": 3,
        }
        mock_engine.storage_size.return_value = {"total_bytes": 512000}
        mock_engine.status.return_value = {"uptime_seconds": 1800}

        result = await service.get_overview()
        assert result.total_sessions == 5
        assert result.total_memories == 50
        assert result.storage_size_bytes == 512000
        assert result.uptime_seconds == 1800

    @pytest.mark.asyncio
    async def test_handles_partial_failure_in_gather(self, service, mock_engine):
        """One failing call should not cancel the others."""
        mock_engine.cache_telemetry.side_effect = Exception("telemetry failed")
        mock_engine.storage_size.return_value = {"total_bytes": 2048}
        mock_engine.status.return_value = {"uptime_seconds": 999}

        result = await service.get_overview()
        assert result.storage_size_bytes == 2048
        assert result.uptime_seconds == 999


class TestAnalyticsServiceGetHealth:
    """Tests for AnalyticsService.get_health."""

    @pytest.mark.asyncio
    async def test_returns_health(self, service, mock_engine):
        mock_engine.status.return_value = {
            "status": "ok",
            "uptime_seconds": 7200,
            "memory_usage_mb": 128.5,
        }
        mock_engine.cache_telemetry.return_value = {"cache_entries": 42}
        mock_engine.storage_size.return_value = {"total_bytes": 2048000}

        result = await service.get_health()
        assert isinstance(result, SystemHealth)
        assert result.status == "ok"
        assert result.uptime_seconds == 7200
        assert result.memory_usage_mb == 128.5
        assert result.storage_size_bytes == 2048000
        assert result.cache_entries == 42

    @pytest.mark.asyncio
    async def test_gathers_independent_calls(self, service, mock_engine):
        """Verify status, telemetry, and storage_size are gathered."""
        mock_engine.status.return_value = {"status": "ok", "uptime_seconds": 100, "memory_usage_mb": 64.0}
        mock_engine.cache_telemetry.return_value = {"cache_entries": 10}
        mock_engine.storage_size.return_value = {"total_bytes": 1024}

        result = await service.get_health()
        assert result.status == "ok"
        assert result.uptime_seconds == 100
        assert result.memory_usage_mb == 64.0
        assert result.cache_entries == 10
        assert result.storage_size_bytes == 1024


class TestAnalyticsServiceGetPerformance:
    """Tests for AnalyticsService.get_performance."""

    @pytest.mark.asyncio
    async def test_returns_performance(self, service, mock_engine):
        mock_engine.cache_telemetry.return_value = {
            "avg_response_time_ms": 15.5,
            "total_operations": 5000,
            "cache_hit_rate": 0.85,
        }
        result = await service.get_performance()
        assert isinstance(result, PerformanceMetrics)
        assert result.avg_response_time_ms == 15.5
        assert result.total_operations == 5000
        assert result.cache_hit_rate == 0.85


class TestAnalyticsServiceGetResources:
    """Tests for AnalyticsService.get_resources."""

    @pytest.mark.asyncio
    async def test_returns_resources(self, service, mock_engine):
        mock_engine.storage_size.return_value = {"total_bytes": 1048576}
        mock_engine.status.return_value = {
            "cpu_percent": 45.0,
            "memory_usage_mb": 256.0,
        }
        mock_engine.cache_telemetry.return_value = {}

        result = await service.get_resources()
        assert isinstance(result, ResourceUsage)
        assert result.cpu_percent == 45.0
        assert result.memory_mb == 256.0
        assert result.storage_mb == 1.0  # 1048576 bytes = 1 MB

    @pytest.mark.asyncio
    async def test_gathers_independent_calls(self, service, mock_engine):
        """Verify storage_size, status, and cache_telemetry are gathered."""
        mock_engine.storage_size.return_value = {"total_bytes": 2097152}
        mock_engine.status.return_value = {"cpu_percent": 60.0, "memory_usage_mb": 512.0}
        mock_engine.cache_telemetry.return_value = {}

        result = await service.get_resources()
        assert result.cpu_percent == 60.0
        assert result.memory_mb == 512.0
        assert result.storage_mb == 2.0

    @pytest.mark.asyncio
    async def test_handles_partial_failure(self, service, mock_engine):
        """One failing call should not cancel the others."""
        mock_engine.storage_size.side_effect = Exception("storage failed")
        mock_engine.status.return_value = {"cpu_percent": 30.0, "memory_usage_mb": 128.0}
        mock_engine.cache_telemetry.return_value = {}

        result = await service.get_resources()
        assert result.cpu_percent == 30.0
        assert result.memory_mb == 128.0


class TestAnalyticsServiceGetCosts:
    """Tests for AnalyticsService.get_costs."""

    @pytest.mark.asyncio
    async def test_returns_default_costs(self, service, mock_engine):
        result = await service.get_costs()
        assert isinstance(result, CostMetrics)
        assert result.total_cost == 0.0
        assert result.cost_by_model == {}


class TestAnalyticsServiceGetServiceStatus:
    """Tests for AnalyticsService.get_service_status."""

    @pytest.mark.asyncio
    async def test_returns_service_status(self, service, mock_engine):
        mock_engine.status.return_value = {
            "status": "ok",
            "latency_ms": 5.0,
        }
        result = await service.get_service_status()
        assert isinstance(result, ServiceStatus)
        assert result.name == "contexter-server"
        assert result.status == "ok"
        assert result.latency_ms == 5.0
