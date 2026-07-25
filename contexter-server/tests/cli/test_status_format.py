"""Tests for CLI status display formatting — f-string interpolation and error reporting."""

import json
from unittest.mock import AsyncMock, patch, MagicMock

import pytest
from click.testing import CliRunner

from contexter_server.cli.main import cli
from contexter_server.cli.status_commands import _format_uptime, _format_bytes


@pytest.fixture
def runner() -> CliRunner:
    return CliRunner()


class TestStatusFStrings:
    """Verify status output uses f-strings (not literal {variable.attr} text)."""

    @patch("contexter_server.cli.status_commands.StorageEngine")
    def test_status_shows_interpolated_values(
        self, mock_storage: AsyncMock, runner: CliRunner
    ) -> None:
        """Status output should contain actual values, not raw template syntax."""
        engine_instance = mock_storage.return_value
        engine_instance.status = AsyncMock(
            return_value={
                "status": "ok",
                "uptime_seconds": 9999,
                "memory_usage_mb": 42.5,
                "cpu_percent": 12.3,
                "latency_ms": 5.0,
            }
        )
        engine_instance.storage_size = AsyncMock(
            return_value={"total_bytes": 2097152}
        )
        engine_instance.cache_telemetry = AsyncMock(
            return_value={
                "total_sessions": 25,
                "total_memories": 200,
                "total_agents": 5,
                "total_skills": 8,
                "cache_entries": 50,
                "avg_response_time_ms": 3.5,
                "total_operations": 10000,
                "cache_hit_rate": 0.85,
            }
        )

        result = runner.invoke(cli, ["status"])

        assert result.exit_code == 0, f"Output: {result.output}"

        # Must NOT contain raw template syntax
        assert "{overview.total_sessions}" not in result.output
        assert "{overview.total_memories}" not in result.output
        assert "{overview.total_agents}" not in result.output
        assert "{performance.avg_response_time_ms" not in result.output
        assert "{resources.cpu_percent" not in result.output

        # Must contain actual interpolated values
        assert "25" in result.output  # total_sessions
        assert "200" in result.output  # total_memories
        assert "5" in result.output  # total_agents
        assert "3.5" in result.output  # avg_response_time_ms


class TestStatusExceptionReporting:
    """Verify status/gc commands log full exceptions and show user-friendly messages."""

    @patch("contexter_server.cli.status_commands.logger")
    @patch("contexter_server.cli.status_commands.StorageEngine")
    def test_gc_logs_exception(
        self,
        mock_storage: AsyncMock,
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
