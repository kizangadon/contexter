"""Tests for CLI status display formatting — f-string interpolation and error reporting.

Engine telemetry shapes mirror what the real Rust engine emits
(verified against ``contexter_core`` bridge.rs / cache metrics.rs):

- ``cache_telemetry()`` -> snake_case  (``entries_by_type``, ``total_ops``)
- ``storage_size()``    -> camelCase   (``total``, ``perCf``, ``walSize``)
- ``status()``          -> ``{status, version, cacheTelemetry: {...}}``

The ``status`` command additionally reads ``version`` from ``status()`` for
display; the analytics domain models do not carry it, so a missing version
degrades to ``"unknown"`` instead of crashing the report.
"""

import re
from unittest.mock import AsyncMock, MagicMock, patch

import pytest
from click.testing import CliRunner

from contexter_server.cli.main import cli
from contexter_server.cli.status_commands import _format_uptime, _format_bytes


# ---------------------------------------------------------------------------
# Real engine telemetry shapes (verified against contexter_core)
# ---------------------------------------------------------------------------


def _real_status(**overrides: object) -> dict:
    payload = {
        "status": "ok",
        "version": "0.1.0",
        "cacheTelemetry": {
            "entriesByType": {"agent": 3, "session": 10, "skill": 5},
            "hitRatio": 0.85,
            "hits": 8500,
            "misses": 1500,
            "totalOps": 10000,
        },
    }
    payload.update(overrides)
    return payload


def _real_telemetry(**overrides: object) -> dict:
    payload = {
        "gets": 10000,
        "hits": 8500,
        "misses": 1500,
        "stores": 5000,
        "invalidations": 10,
        "total_ops": 10000,
        "entries_by_type": {"session": 60, "memory": 63},
    }
    payload.update(overrides)
    return payload


def _real_storage(**overrides: object) -> dict:
    payload = {
        "perCf": {"agents": 1048576, "sessions": 1048576},
        "total": 2097152,
        "walSize": 0,
    }
    payload.update(overrides)
    return payload


def _wire_status_engine(
    mock_storage: MagicMock,
    status: object | None = None,
    telemetry: object | None = None,
    storage: object | None = None,
    sessions: int = 25,
    memories: int = 200,
    agents: int = 7,
    skills: int = 9,
) -> None:
    """Configure a StorageEngine mock to return the real engine shapes."""
    engine_instance = mock_storage.return_value
    engine_instance.status = AsyncMock(return_value=_real_status() if status is None else status)
    engine_instance.cache_telemetry = AsyncMock(
        return_value=_real_telemetry() if telemetry is None else telemetry
    )
    engine_instance.storage_size = AsyncMock(
        return_value=_real_storage() if storage is None else storage
    )
    engine_instance.count_sessions = AsyncMock(return_value=sessions)
    engine_instance.count_memories = AsyncMock(return_value=memories)
    engine_instance.count_agents = AsyncMock(return_value=agents)
    engine_instance.count_skills = AsyncMock(return_value=skills)


@pytest.fixture
def runner() -> CliRunner:
    return CliRunner()


class TestStatusFStrings:
    """Verify status output uses f-strings (not literal {variable.attr} text)."""

    @patch("contexter_server.cli.status_commands.StorageEngine")
    def test_status_shows_interpolated_values(
        self, mock_storage: MagicMock, runner: CliRunner
    ) -> None:
        """Status output should contain real engine values, not raw template syntax."""
        _wire_status_engine(mock_storage)

        result = runner.invoke(cli, ["status"])

        assert result.exit_code == 0, f"Output: {result.output}"

        # Must NOT contain raw template syntax
        assert "{overview.total_sessions}" not in result.output
        assert "{overview.total_memories}" not in result.output
        assert "{overview.total_agents}" not in result.output
        assert "{performance.avg_response_time_ms" not in result.output
        assert "{resources.cpu_percent" not in result.output

        # Must contain actual interpolated values from the real engine shapes
        assert "ok" in result.output  # status()["status"]
        assert "0.1.0" in result.output  # status()["version"]
        assert "25" in result.output  # count_sessions
        assert "200" in result.output  # count_memories
        assert re.search(r"Agents:\s+7", result.output)  # count_agents
        assert re.search(r"Skills:\s+9", result.output)  # count_skills
        assert "123" in result.output  # sum(cache_telemetry entries_by_type)
        assert "10000" in result.output  # cache_telemetry total_ops
        assert "85.0%" in result.output  # hits / (hits + misses)
        assert "2.0 MB" in result.output  # storage total / MB

    @patch("contexter_server.cli.status_commands.StorageEngine")
    def test_status_degrades_gracefully_without_optional_keys(
        self, mock_storage: MagicMock, runner: CliRunner
    ) -> None:
        """Missing optional telemetry/version keys -> sane fallbacks, no crash."""
        _wire_status_engine(
            mock_storage,
            status={},  # no cacheTelemetry, no version, no health keys
            telemetry={},
            storage={},
            sessions=0,
            memories=0,
            agents=0,
            skills=0,
        )

        result = runner.invoke(cli, ["status"])

        assert result.exit_code == 0, f"Output: {result.output}"
        assert "Contexter System Status" in result.output
        assert "Uptime:" in result.output
        assert "0s" in result.output  # graceful uptime fallback
        assert "Version:" in result.output
        assert "unknown" in result.output  # version safe fallback
        assert "Traceback" not in result.output

    @patch("contexter_server.cli.status_commands.StorageEngine")
    def test_status_version_missing_renders_unknown(
        self, mock_storage: MagicMock, runner: CliRunner
    ) -> None:
        """Real payload without a version key -> safe fallback, no crash."""
        _wire_status_engine(
            mock_storage,
            status={
                "status": "ok",
                "cacheTelemetry": {
                    "entriesByType": {},
                    "hitRatio": 0.0,
                    "hits": 0,
                    "misses": 0,
                    "totalOps": 0,
                },
            },
        )

        result = runner.invoke(cli, ["status"])

        assert result.exit_code == 0, f"Output: {result.output}"
        assert "Version:" in result.output
        assert "unknown" in result.output


class TestStatusExceptionReporting:
    """Verify status/gc commands log full exceptions and show user-friendly messages."""

    @patch("contexter_server.cli.status_commands.logger")
    @patch("contexter_server.cli.status_commands.StorageEngine")
    def test_gc_logs_exception(
        self,
        mock_storage: MagicMock,
        mock_logger: MagicMock,
        runner: CliRunner,
    ) -> None:
        """GC command should log full exception via logger.exception()."""
        engine_instance = mock_storage.return_value
        engine_instance.flush = AsyncMock(
            side_effect=RuntimeError("internal db error")
        )

        # Need to patch the logger in status_commands module
        with patch(
            "contexter_server.cli.status_commands.logger.exception"
        ) as mock_log_exc:
            result = runner.invoke(cli, ["gc"])

            assert result.exit_code != 0
            mock_log_exc.assert_called_once()


class TestFormatHelpers:
    """Tests for _format_uptime and _format_bytes."""

    def test_format_uptime(self):
        assert _format_uptime(0) == "0s"
        assert _format_uptime(30) == "30s"
        assert _format_uptime(120) == "2m 0s"
        assert _format_uptime(3600) == "1h 0s"  # minutes=0 is skipped
        assert _format_uptime(90061) == "1d 1h 1m 1s"

    def test_format_bytes(self):
        assert _format_bytes(0) == "0.0 B"
        assert _format_bytes(1023) == "1023.0 B"
        assert _format_bytes(1024) == "1.0 KB"
        assert _format_bytes(1048576) == "1.0 MB"
        assert _format_bytes(1073741824) == "1.0 GB"
