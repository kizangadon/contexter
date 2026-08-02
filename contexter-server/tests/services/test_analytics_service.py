"""Tests for AnalyticsService.

The engine telemetry shapes used here mirror what the real Rust engine emits
(verified live against ``contexter_core``):

- ``cache_telemetry()`` -> snake_case  (``entries_by_type``, ``total_ops``)
- ``storage_size()``    -> camelCase   (``total``, ``perCf``, ``walSize``)
- ``status()``          -> ``{status, version, cacheTelemetry: {...}}``

Legacy tests mocked snake_case keys the engine never returns
(``total_sessions``, ``total_bytes``, ``uptime_seconds``, ...) which encoded
the telemetry-mapping defect this suite guards against.
"""

import logging
import re
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

_ANALYTICS_LOGGER = "contexter_server.services.analytics_service"

# structlog's ConsoleRenderer interleaves ANSI color codes into the rendered
# message; strip them before asserting on log content.
_ANSI_ESCAPE = re.compile(r"\x1b\[[0-9;]*m")


# ---------------------------------------------------------------------------
# Real engine telemetry shapes (live-verified against contexter_core)
# ---------------------------------------------------------------------------


def _real_telemetry(**overrides):
    payload = {
        "gets": 10,
        "hits": 8,
        "misses": 2,
        "stores": 15,
        "invalidations": 3,
        "total_ops": 30,
        "entries_by_type": {"agent": 1, "session": 1, "skill": 1},
    }
    payload.update(overrides)
    return payload


def _real_storage(**overrides):
    payload = {
        "perCf": {"agents": 2048, "sessions": 2048, "skills": 2048},
        "total": 24576,
        "walSize": 0,
    }
    payload.update(overrides)
    return payload


def _real_status(**overrides):
    payload = {
        "status": "ok",
        "version": "0.1.0",
        "cacheTelemetry": {
            "entriesByType": {"agent": 1, "session": 1, "skill": 1},
            "hitRatio": 0.0,
            "hits": 0,
            "misses": 0,
            "totalOps": 0,
        },
    }
    payload.update(overrides)
    return payload


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
    async def test_returns_overview_from_real_telemetry(self, service, mock_engine):
        mock_engine.storage_size.return_value = _real_storage(total=1024000)
        mock_engine.status.return_value = _real_status()
        mock_engine.count_sessions.return_value = 10
        mock_engine.count_memories.return_value = 100
        mock_engine.count_agents.return_value = 3
        mock_engine.count_skills.return_value = 5

        result = await service.get_overview()

        assert isinstance(result, AnalyticsOverview)
        assert result.total_sessions == 10
        assert result.total_memories == 100
        assert result.total_agents == 3
        assert result.total_skills == 5
        assert result.storage_size_bytes == 1024000
        assert result.uptime_seconds == 0  # no engine source -> graceful default

    @pytest.mark.asyncio
    async def test_returns_defaults_on_empty_telemetry(self, service, mock_engine):
        mock_engine.storage_size.return_value = {}
        mock_engine.status.return_value = {}
        mock_engine.count_sessions.return_value = 0
        mock_engine.count_memories.return_value = 0
        mock_engine.count_agents.return_value = 0
        mock_engine.count_skills.return_value = 0

        result = await service.get_overview()

        assert result.total_sessions == 0
        assert result.storage_size_bytes == 0

    @pytest.mark.asyncio
    async def test_gathers_independent_calls(self, service, mock_engine):
        """Verify all six store/telemetry calls are awaited concurrently."""
        mock_engine.storage_size.return_value = _real_storage(total=512000)
        mock_engine.status.return_value = _real_status()
        mock_engine.count_sessions.return_value = 5
        mock_engine.count_memories.return_value = 50
        mock_engine.count_agents.return_value = 2
        mock_engine.count_skills.return_value = 3

        result = await service.get_overview()

        assert result.total_sessions == 5
        assert result.total_memories == 50
        assert result.total_agents == 2
        assert result.total_skills == 3
        assert result.storage_size_bytes == 512000
        mock_engine.storage_size.assert_awaited_once()
        mock_engine.status.assert_awaited_once()
        mock_engine.count_sessions.assert_awaited_once()
        mock_engine.count_memories.assert_awaited_once()
        mock_engine.count_agents.assert_awaited_once()
        mock_engine.count_skills.assert_awaited_once()
        mock_engine.cache_telemetry.assert_not_awaited()  # overview reads the store, not the cache

    @pytest.mark.asyncio
    async def test_uses_dedicated_counts_not_full_store_scan(self, service, mock_engine):
        """AC-ACE-002: get_overview must call count_agents/count_skills and
        MUST NOT call list_agents/list_skills (the O(store) full scan)."""
        mock_engine.storage_size.return_value = _real_storage(total=2048)
        mock_engine.status.return_value = _real_status()
        mock_engine.count_sessions.return_value = 1
        mock_engine.count_memories.return_value = 1
        mock_engine.count_agents.return_value = 3
        mock_engine.count_skills.return_value = 2

        result = await service.get_overview()

        assert result.total_agents == 3
        assert result.total_skills == 2
        mock_engine.count_agents.assert_awaited_once()
        mock_engine.count_skills.assert_awaited_once()
        mock_engine.list_agents.assert_not_awaited()
        mock_engine.list_skills.assert_not_awaited()

    @pytest.mark.asyncio
    async def test_handles_partial_failure_in_gather(self, service, mock_engine):
        """One failing call should not cancel the others."""
        mock_engine.storage_size.return_value = _real_storage(total=2048)
        mock_engine.status.return_value = _real_status()
        mock_engine.count_sessions.return_value = 1
        mock_engine.count_memories.side_effect = Exception("memories failed")
        mock_engine.count_agents.return_value = 1
        mock_engine.count_skills.return_value = 1

        result = await service.get_overview()

        assert result.storage_size_bytes == 2048
        assert result.total_sessions == 1
        assert result.total_memories == 0  # failed call -> logged default

    @pytest.mark.asyncio
    async def test_logs_missing_keys_explicitly(self, service, mock_engine, caplog):
        """REQ-AN-003 + REQ-SH-001: missing keys are logged at DEBUG — never
        silently defaulted, never a WARNING on the success path."""
        mock_engine.storage_size.return_value = {"perCf": {}}  # "total" absent
        mock_engine.status.return_value = {}
        mock_engine.count_sessions.return_value = 0
        mock_engine.count_memories.return_value = 0
        mock_engine.count_agents.return_value = 0
        mock_engine.count_skills.return_value = 0

        with caplog.at_level(logging.DEBUG, logger=_ANALYTICS_LOGGER):
            result = await service.get_overview()

        assert result.storage_size_bytes == 0
        # structlog's stdlib LoggerFactory renders the full line (event +
        # key-value pairs) before the stdlib call, so assert on the message.
        missing_key = [
            r
            for r in caplog.records
            if r.name == _ANALYTICS_LOGGER
            and "analytics.missing_key" in _ANSI_ESCAPE.sub("", r.getMessage())
        ]
        assert missing_key, "analytics.missing_key must still be logged (signal not lost)"
        assert all(
            r.levelno == logging.DEBUG for r in missing_key
        ), "analytics.missing_key must be DEBUG, not WARNING (REQ-SH-001)"
        assert any(
            "key=total" in _ANSI_ESCAPE.sub("", r.getMessage()) for r in missing_key
        )

    @pytest.mark.asyncio
    async def test_success_path_emits_no_warnings(self, service, mock_engine, caplog):
        """AC-SH-001: get_overview with imperfect telemetry (missing keys,
        non-integer counts, failed gather) emits ZERO WARNING+ records."""
        mock_engine.storage_size.return_value = {"perCf": {}}  # "total" absent
        mock_engine.status.return_value = {}
        mock_engine.count_sessions.return_value = "10"  # non-integer count
        mock_engine.count_memories.side_effect = Exception("memories failed")
        mock_engine.count_agents.return_value = 0
        mock_engine.count_skills.return_value = 0

        with caplog.at_level(logging.INFO, logger=_ANALYTICS_LOGGER):
            result = await service.get_overview()

        assert result.total_sessions == 0  # graceful defaults still apply
        assert result.total_memories == 0
        assert result.storage_size_bytes == 0
        warnings = [
            r
            for r in caplog.records
            if r.name == _ANALYTICS_LOGGER and r.levelno >= logging.WARNING
        ]
        assert warnings == [], (
            f"success path must emit zero WARNING+ records, got: "
            f"{[_ANSI_ESCAPE.sub('', r.getMessage()) for r in warnings]}"
        )

    @pytest.mark.asyncio
    async def test_non_integer_count_logged_at_debug(self, service, mock_engine, caplog):
        """EC-SH-004: the per-call non-integer-count signal is DEBUG, not WARNING."""
        mock_engine.storage_size.return_value = _real_storage()
        mock_engine.status.return_value = _real_status()
        mock_engine.count_sessions.return_value = "10"
        mock_engine.count_memories.return_value = 0
        mock_engine.count_agents.return_value = 0
        mock_engine.count_skills.return_value = 0

        with caplog.at_level(logging.DEBUG, logger=_ANALYTICS_LOGGER):
            result = await service.get_overview()

        assert result.total_sessions == 0  # graceful default
        records = [
            r
            for r in caplog.records
            if r.name == _ANALYTICS_LOGGER
            and "analytics.non_integer_count" in _ANSI_ESCAPE.sub("", r.getMessage())
        ]
        assert records, "analytics.non_integer_count must still be logged (signal not lost)"
        assert all(
            r.levelno == logging.DEBUG for r in records
        ), "analytics.non_integer_count must be DEBUG, not WARNING (EC-SH-004)"

    @pytest.mark.asyncio
    async def test_invalid_entries_by_type_logged_at_debug(
        self, service, mock_engine, caplog
    ):
        """EC-SH-004: the per-call invalid-entries signal is DEBUG, not WARNING."""
        mock_engine.status.return_value = _real_status()
        mock_engine.cache_telemetry.return_value = {"entries_by_type": "n/a"}
        mock_engine.storage_size.return_value = _real_storage(total=1024)

        with caplog.at_level(logging.DEBUG, logger=_ANALYTICS_LOGGER):
            result = await service.get_health()

        assert result.cache_entries == 0  # graceful default
        records = [
            r
            for r in caplog.records
            if r.name == _ANALYTICS_LOGGER
            and "analytics.invalid_entries_by_type"
            in _ANSI_ESCAPE.sub("", r.getMessage())
        ]
        assert records, (
            "analytics.invalid_entries_by_type must still be logged (signal not lost)"
        )
        assert all(
            r.levelno == logging.DEBUG for r in records
        ), "analytics.invalid_entries_by_type must be DEBUG, not WARNING (EC-SH-004)"

    @pytest.mark.asyncio
    async def test_maps_uptime_when_engine_provides_it(self, service, mock_engine):
        """Forward-compatible: if the engine ever emits uptime, it is mapped."""
        mock_engine.storage_size.return_value = _real_storage()
        mock_engine.status.return_value = _real_status(uptime_seconds=3600)
        mock_engine.count_sessions.return_value = 1
        mock_engine.count_memories.return_value = 1
        mock_engine.count_agents.return_value = 1
        mock_engine.count_skills.return_value = 1

        result = await service.get_overview()

        assert result.uptime_seconds == 3600


class TestAnalyticsServiceGetHealth:
    """Tests for AnalyticsService.get_health."""

    @pytest.mark.asyncio
    async def test_returns_health_from_real_telemetry(self, service, mock_engine):
        mock_engine.status.return_value = _real_status()
        mock_engine.cache_telemetry.return_value = _real_telemetry()
        mock_engine.storage_size.return_value = _real_storage(total=2048000)

        result = await service.get_health()

        assert isinstance(result, SystemHealth)
        assert result.status == "ok"
        assert result.storage_size_bytes == 2048000
        assert result.cache_entries == 3  # agent + session + skill cache-resident
        assert result.uptime_seconds == 0  # no engine source -> graceful default
        assert result.memory_usage_mb == 0.0

    @pytest.mark.asyncio
    async def test_gathers_independent_calls(self, service, mock_engine):
        """Verify status, telemetry, and storage_size are gathered."""
        mock_engine.status.return_value = _real_status()
        mock_engine.cache_telemetry.return_value = _real_telemetry(
            entries_by_type={"agent": 5, "session": 2}
        )
        mock_engine.storage_size.return_value = _real_storage(total=1024)

        result = await service.get_health()

        assert result.status == "ok"
        assert result.cache_entries == 7
        assert result.storage_size_bytes == 1024
        mock_engine.status.assert_awaited_once()
        mock_engine.cache_telemetry.assert_awaited_once()
        mock_engine.storage_size.assert_awaited_once()


class TestAnalyticsServiceGetPerformance:
    """Tests for AnalyticsService.get_performance."""

    @pytest.mark.asyncio
    async def test_returns_performance_from_real_telemetry(self, service, mock_engine):
        mock_engine.cache_telemetry.return_value = _real_telemetry(
            gets=100, hits=85, misses=15, total_ops=5000
        )

        result = await service.get_performance()

        assert isinstance(result, PerformanceMetrics)
        assert result.total_operations == 5000  # engine "total_ops"
        assert result.cache_hit_rate == pytest.approx(0.85)  # hits / (hits + misses)
        assert result.avg_response_time_ms == 0.0  # no engine source -> graceful default

    @pytest.mark.asyncio
    async def test_cache_hit_rate_zero_when_no_attempts(self, service, mock_engine):
        mock_engine.cache_telemetry.return_value = _real_telemetry(
            gets=0, hits=0, misses=0, total_ops=0
        )

        result = await service.get_performance()

        assert result.cache_hit_rate == 0.0


class TestAnalyticsServiceGetResources:
    """Tests for AnalyticsService.get_resources."""

    @pytest.mark.asyncio
    async def test_returns_resources_from_real_telemetry(self, service, mock_engine):
        mock_engine.storage_size.return_value = _real_storage(total=1048576)
        mock_engine.status.return_value = _real_status()
        mock_engine.cache_telemetry.return_value = _real_telemetry()

        result = await service.get_resources()

        assert isinstance(result, ResourceUsage)
        assert result.storage_mb == 1.0  # 1048576 bytes = 1 MB
        assert result.cpu_percent == 0.0  # no engine source -> graceful default
        assert result.memory_mb == 0.0

    @pytest.mark.asyncio
    async def test_handles_partial_failure(self, service, mock_engine):
        """One failing call should not cancel the others."""
        mock_engine.storage_size.side_effect = Exception("storage failed")
        mock_engine.status.return_value = _real_status()
        mock_engine.cache_telemetry.return_value = {}

        result = await service.get_resources()

        assert result.storage_mb == 0.0


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
        mock_engine.status.return_value = _real_status()

        result = await service.get_service_status()

        assert isinstance(result, ServiceStatus)
        assert result.name == "contexter-server"
        assert result.status == "ok"
        assert result.latency_ms == 0.0  # no engine source -> graceful default

    @pytest.mark.asyncio
    async def test_maps_latency_when_engine_provides_it(self, service, mock_engine):
        mock_engine.status.return_value = _real_status(latency_ms=5.0)

        result = await service.get_service_status()

        assert result.latency_ms == 5.0
