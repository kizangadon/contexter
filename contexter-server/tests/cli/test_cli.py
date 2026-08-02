"""Tests for the Contexter CLI using Click's CliRunner.

Tests mock the StorageEngine at the bridge level to avoid requiring a
running Rust engine during unit tests.
"""

import json
from unittest.mock import AsyncMock, patch

import pytest
from click.testing import CliRunner

from contexter_server.cli.main import cli


# ---------------------------------------------------------------------------
# Fixtures
# ---------------------------------------------------------------------------


@pytest.fixture
def runner() -> CliRunner:
    """Provide a Click CliRunner for invoking CLI commands."""
    return CliRunner()


@pytest.fixture
def mock_engine() -> AsyncMock:
    """Return a mock StorageEngine that replaces the real bridge."""
    return AsyncMock()


# ---------------------------------------------------------------------------
# contexter --help
# ---------------------------------------------------------------------------


class TestCliHelp:
    """Verify the CLI group structure and help output."""

    def test_cli_help_shows_all_commands(self, runner: CliRunner) -> None:
        result = runner.invoke(cli, ["--help"])
        assert result.exit_code == 0
        assert "Contexter" in result.output
        for cmd in ("session", "memory", "status", "export", "gc"):
            assert cmd in result.output

    def test_session_help_shows_subcommands(self, runner: CliRunner) -> None:
        result = runner.invoke(cli, ["session", "--help"])
        assert result.exit_code == 0
        for sub in ("create", "list", "get", "delete"):
            assert sub in result.output

    def test_memory_help_shows_subcommands(self, runner: CliRunner) -> None:
        result = runner.invoke(cli, ["memory", "--help"])
        assert result.exit_code == 0
        for sub in ("create", "search"):
            assert sub in result.output

    def test_status_help(self, runner: CliRunner) -> None:
        result = runner.invoke(cli, ["status", "--help"])
        assert result.exit_code == 0

    def test_export_help(self, runner: CliRunner) -> None:
        result = runner.invoke(cli, ["export", "--help"])
        assert result.exit_code == 0

    def test_gc_help(self, runner: CliRunner) -> None:
        result = runner.invoke(cli, ["gc", "--help"])
        assert result.exit_code == 0


# ---------------------------------------------------------------------------
# contexter session create
# ---------------------------------------------------------------------------


class TestSessionCreate:
    """Tests for `contexter session create`."""

    @patch("contexter_server.cli.session_commands.StorageEngine")
    def test_creates_session(
        self, mock_storage: AsyncMock, runner: CliRunner, any_uuid: str
    ) -> None:
        engine_instance = mock_storage.return_value
        engine_instance.create_session = AsyncMock(
            return_value={
                "id": any_uuid,
                "agent_id": any_uuid,
                "project": "test-project",
                "name": "My Session",
                "status": "active",
                "started_at": "2026-07-25T00:00:00+00:00",
                "updated_at": "2026-07-25T00:00:00+00:00",
            }
        )

        result = runner.invoke(
            cli,
            [
                "session",
                "create",
                "--agent-id", any_uuid,
                "--project", "test-project",
                "--name", "My Session",
            ],
        )

        assert result.exit_code == 0, f"Exit code: {result.exit_code}, output: {result.output}"
        assert "Session created" in result.output
        assert any_uuid in result.output
        assert "test-project" in result.output

    @patch("contexter_server.cli.session_commands.StorageEngine")
    def test_create_requires_agent_id(self, mock_storage: AsyncMock, runner: CliRunner) -> None:
        result = runner.invoke(cli, ["session", "create", "--project", "test"])
        assert result.exit_code != 0
        assert "--agent-id" in result.output

    @patch("contexter_server.cli.session_commands.StorageEngine")
    def test_create_requires_project(self, mock_storage: AsyncMock, runner: CliRunner) -> None:
        result = runner.invoke(cli, ["session", "create", "--agent-id", "00000000-0000-0000-0000-000000000001"])
        assert result.exit_code != 0
        assert "--project" in result.output


# ---------------------------------------------------------------------------
# contexter session list
# ---------------------------------------------------------------------------


class TestSessionList:
    """Tests for `contexter session list`."""

    @patch("contexter_server.cli.session_commands.StorageEngine")
    def test_lists_sessions(self, mock_storage: AsyncMock, runner: CliRunner, any_uuid: str) -> None:
        engine_instance = mock_storage.return_value
        engine_instance.list_sessions = AsyncMock(
            return_value=[
                {
                    "id": any_uuid,
                    "agent_id": any_uuid,
                    "project": "p1",
                    "name": "Session 1",
                    "status": "active",
                    "started_at": "2026-07-25T00:00:00+00:00",
                    "updated_at": "2026-07-25T00:00:00+00:00",
                },
                {
                    "id": any_uuid.replace("1", "2"),
                    "agent_id": any_uuid,
                    "project": "p2",
                    "name": "Session 2",
                    "status": "completed",
                    "started_at": "2026-07-24T00:00:00+00:00",
                    "updated_at": "2026-07-24T00:00:00+00:00",
                    "completed_at": "2026-07-24T12:00:00+00:00",
                },
            ]
        )

        result = runner.invoke(cli, ["session", "list"])

        assert result.exit_code == 0, f"Output: {result.output}"
        assert "Sessions (2)" in result.output
        assert "p1" in result.output
        assert "p2" in result.output

    @patch("contexter_server.cli.session_commands.StorageEngine")
    def test_lists_with_filters(self, mock_storage: AsyncMock, runner: CliRunner, any_uuid: str) -> None:
        engine_instance = mock_storage.return_value
        engine_instance.list_sessions = AsyncMock(
            return_value=[
                {
                    "id": any_uuid,
                    "agent_id": any_uuid,
                    "project": "my-project",
                    "name": None,
                    "status": "active",
                    "started_at": "2026-07-25T00:00:00+00:00",
                    "updated_at": "2026-07-25T00:00:00+00:00",
                }
            ]
        )

        result = runner.invoke(cli, ["session", "list", "--project", "my-project", "--status", "active"])

        assert result.exit_code == 0
        assert "my-project" in result.output

    @patch("contexter_server.cli.session_commands.StorageEngine")
    def test_list_empty(self, mock_storage: AsyncMock, runner: CliRunner) -> None:
        engine_instance = mock_storage.return_value
        engine_instance.list_sessions = AsyncMock(return_value=[])

        result = runner.invoke(cli, ["session", "list"])

        assert result.exit_code == 0
        assert "No sessions found" in result.output

    @patch("contexter_server.cli.session_commands.StorageEngine")
    def test_list_json_output(self, mock_storage: AsyncMock, runner: CliRunner, any_uuid: str) -> None:
        engine_instance = mock_storage.return_value
        engine_instance.list_sessions = AsyncMock(
            return_value=[
                {
                    "id": any_uuid,
                    "agent_id": any_uuid,
                    "project": "p1",
                    "name": "S1",
                    "status": "active",
                    "started_at": "2026-07-25T00:00:00+00:00",
                    "updated_at": "2026-07-25T00:00:00+00:00",
                }
            ]
        )

        result = runner.invoke(cli, ["session", "list", "--json"])

        assert result.exit_code == 0
        parsed = json.loads(result.output)
        assert isinstance(parsed, list)
        assert len(parsed) == 1
        assert parsed[0]["project"] == "p1"


# ---------------------------------------------------------------------------
# contexter session get
# ---------------------------------------------------------------------------


class TestSessionGet:
    """Tests for `contexter session get`."""

    @patch("contexter_server.cli.session_commands.StorageEngine")
    def test_gets_session(self, mock_storage: AsyncMock, runner: CliRunner, any_uuid: str) -> None:
        engine_instance = mock_storage.return_value
        engine_instance.get_session = AsyncMock(
            return_value={
                "id": any_uuid,
                "agent_id": any_uuid,
                "project": "test-project",
                "name": "Test",
                "status": "active",
                "started_at": "2026-07-25T00:00:00+00:00",
                "updated_at": "2026-07-25T00:00:00+00:00",
            }
        )

        result = runner.invoke(cli, ["session", "get", any_uuid])

        assert result.exit_code == 0
        assert any_uuid in result.output

    @patch("contexter_server.cli.session_commands.StorageEngine")
    def test_get_returns_error_on_missing(self, mock_storage: AsyncMock, runner: CliRunner) -> None:
        engine_instance = mock_storage.return_value
        engine_instance.get_session = AsyncMock(return_value=None)

        result = runner.invoke(cli, ["session", "get", "nonexistent-id"])

        assert result.exit_code != 0
        assert "not found" in result.output


# ---------------------------------------------------------------------------
# contexter session delete
# ---------------------------------------------------------------------------


class TestSessionDelete:
    """Tests for `contexter session delete`."""

    @patch("contexter_server.cli.session_commands.StorageEngine")
    def test_deletes_session(self, mock_storage: AsyncMock, runner: CliRunner, any_uuid: str) -> None:
        engine_instance = mock_storage.return_value
        engine_instance.delete_session = AsyncMock(return_value=None)

        result = runner.invoke(cli, ["session", "delete", any_uuid])

        assert result.exit_code == 0
        assert "deleted" in result.output


# ---------------------------------------------------------------------------
# contexter memory create
# ---------------------------------------------------------------------------


class TestMemoryCreate:
    """Tests for `contexter memory create`."""

    @patch("contexter_server.cli.memory_commands.StorageEngine")
    def test_creates_memory(
        self, mock_storage: AsyncMock, runner: CliRunner, any_uuid: str
    ) -> None:
        engine_instance = mock_storage.return_value
        engine_instance.create_memory = AsyncMock(
            return_value={
                "id": any_uuid,
                "session_id": any_uuid,
                "agent_id": any_uuid,
                "role": "user",
                "content": "Hello, world!",
                "tokens": 5,
                "created_at": "2026-07-25T00:00:00+00:00",
            }
        )

        result = runner.invoke(
            cli,
            [
                "memory",
                "create",
                "--session-id", any_uuid,
                "--agent-id", any_uuid,
                "--role", "user",
                "--content", "Hello, world!",
            ],
        )

        assert result.exit_code == 0, f"Output: {result.output}"
        assert "Memory created" in result.output
        assert "Hello, world!" in result.output

    @patch("contexter_server.cli.memory_commands.StorageEngine")
    def test_create_validates_role(self, mock_storage: AsyncMock, runner: CliRunner, any_uuid: str) -> None:
        result = runner.invoke(
            cli,
            [
                "memory",
                "create",
                "--session-id", any_uuid,
                "--agent-id", any_uuid,
                "--role", "invalid_role",
                "--content", "test",
            ],
        )

        assert result.exit_code != 0
        assert "invalid_role" in result.output


# ---------------------------------------------------------------------------
# contexter memory search
# ---------------------------------------------------------------------------


class TestMemorySearch:
    """Tests for `contexter memory search`."""

    @patch("contexter_server.cli.memory_commands.StorageEngine")
    def test_search_memories(self, mock_storage: AsyncMock, runner: CliRunner, any_uuid: str) -> None:
        engine_instance = mock_storage.return_value
        engine_instance.search_memories = AsyncMock(
            return_value=[
                {"id": any_uuid, "content": "found it", "score": 0.95, "session_id": any_uuid},
            ]
        )
        engine_instance.count_memories = AsyncMock(return_value=1)

        result = runner.invoke(cli, ["memory", "search", "test query"])

        assert result.exit_code == 0, f"Output: {result.output}"
        assert "Results" in result.output
        assert "found it" in result.output

    @patch("contexter_server.cli.memory_commands.StorageEngine")
    def test_search_empty(self, mock_storage: AsyncMock, runner: CliRunner) -> None:
        engine_instance = mock_storage.return_value
        engine_instance.search_memories = AsyncMock(return_value=[])
        engine_instance.count_memories = AsyncMock(return_value=0)

        result = runner.invoke(cli, ["memory", "search", "nonexistent"])
        assert result.exit_code == 0
        assert "No results" in result.output

    @patch("contexter_server.cli.memory_commands.StorageEngine")
    def test_search_json_output(self, mock_storage: AsyncMock, runner: CliRunner, any_uuid: str) -> None:
        engine_instance = mock_storage.return_value
        engine_instance.search_memories = AsyncMock(
            return_value=[{"id": any_uuid, "content": "data", "score": 0.9}]
        )
        engine_instance.count_memories = AsyncMock(return_value=1)

        result = runner.invoke(cli, ["memory", "search", "query", "--json"])

        assert result.exit_code == 0
        parsed = json.loads(result.output)
        assert "results" in parsed
        assert parsed["total"] == 1

    @patch("contexter_server.cli.memory_commands.StorageEngine")
    def test_search_with_options(self, mock_storage: AsyncMock, runner: CliRunner, any_uuid: str) -> None:
        engine_instance = mock_storage.return_value
        engine_instance.search_memories = AsyncMock(return_value=[])
        engine_instance.count_memories = AsyncMock(return_value=0)

        result = runner.invoke(
            cli,
            [
                "memory",
                "search",
                "query",
                "--type", "memory",
                "--project", "my-project",
                "--limit", "10",
            ],
        )

        assert result.exit_code == 0


# ---------------------------------------------------------------------------
# contexter status
# ---------------------------------------------------------------------------


class TestStatus:
    """Tests for `contexter status`."""

    @patch("contexter_server.cli.status_commands.StorageEngine")
    def test_status_shows_system_info(self, mock_storage: AsyncMock, runner: CliRunner) -> None:
        engine_instance = mock_storage.return_value

        # Wire up all the async methods the status command calls. Shapes
        # mirror the real Rust engine (snake_case cache_telemetry, camelCase
        # storage_size, nested status cacheTelemetry).
        engine_instance.status = AsyncMock(
            return_value={
                "status": "ok",
                "version": "0.1.0",
                "cacheTelemetry": {
                    "entriesByType": {"agent": 3, "session": 10, "skill": 5},
                    "hitRatio": 0.0,
                    "hits": 0,
                    "misses": 0,
                    "totalOps": 5000,
                },
            }
        )
        engine_instance.storage_size = AsyncMock(
            return_value={"perCf": {}, "total": 1048576, "walSize": 0}
        )
        engine_instance.cache_telemetry = AsyncMock(
            return_value={
                "gets": 1000,
                "hits": 850,
                "misses": 150,
                "stores": 15,
                "invalidations": 3,
                "total_ops": 5000,
                "entries_by_type": {"agent": 3, "session": 10, "skill": 5},
            }
        )
        engine_instance.count_sessions = AsyncMock(return_value=10)
        engine_instance.count_memories = AsyncMock(return_value=100)
        engine_instance.count_agents = AsyncMock(return_value=3)
        engine_instance.count_skills = AsyncMock(return_value=5)

        result = runner.invoke(cli, ["status"])

        assert result.exit_code == 0, f"Output: {result.output}"
        assert "Contexter System Status" in result.output
        assert "ok" in result.output


# ---------------------------------------------------------------------------
# contexter export
# ---------------------------------------------------------------------------


class TestExport:
    """Tests for `contexter export`."""

    @patch("contexter_server.cli.export_commands.StorageEngine")
    def test_export_json_default(self, mock_storage: AsyncMock, runner: CliRunner, any_uuid: str) -> None:
        engine_instance = mock_storage.return_value
        # ExportService.submit calls these bridge methods
        engine_instance.list_sessions = AsyncMock(return_value=[])
        engine_instance.search_memories = AsyncMock(return_value=[])
        engine_instance.list_agents = AsyncMock(return_value=[])
        engine_instance.list_skills = AsyncMock(return_value=[])
        engine_instance.set_setting = AsyncMock()
        engine_instance.get_setting = AsyncMock(return_value=None)

        result = runner.invoke(cli, ["export"])

        assert result.exit_code == 0, f"Output: {result.output}"
        assert "Export submitted" in result.output
        assert "json" in result.output

    @patch("contexter_server.cli.export_commands.StorageEngine")
    def test_export_with_entities(self, mock_storage: AsyncMock, runner: CliRunner) -> None:
        engine_instance = mock_storage.return_value
        engine_instance.list_sessions = AsyncMock(return_value=[])
        engine_instance.search_memories = AsyncMock(return_value=[])
        engine_instance.set_setting = AsyncMock()
        engine_instance.get_setting = AsyncMock(return_value=None)

        result = runner.invoke(
            cli,
            [
                "export",
                "--format", "yaml",
                "--entities", "sessions,memories",
            ],
        )

        assert result.exit_code == 0
        assert "yaml" in result.output

    @patch("contexter_server.cli.export_commands.StorageEngine")
    def test_export_json_output(self, mock_storage: AsyncMock, runner: CliRunner) -> None:
        engine_instance = mock_storage.return_value
        engine_instance.list_sessions = AsyncMock(return_value=[])
        engine_instance.search_memories = AsyncMock(return_value=[])
        engine_instance.list_agents = AsyncMock(return_value=[])
        engine_instance.list_skills = AsyncMock(return_value=[])
        engine_instance.set_setting = AsyncMock()
        engine_instance.get_setting = AsyncMock(return_value=None)

        result = runner.invoke(cli, ["export", "--json"])

        assert result.exit_code == 0
        parsed = json.loads(result.output)
        assert "status" in parsed

    @patch("contexter_server.cli.export_commands.StorageEngine")
    def test_export_validates_format(self, mock_storage: AsyncMock, runner: CliRunner) -> None:
        result = runner.invoke(cli, ["export", "--format", "xml"])
        assert result.exit_code != 0
        assert "xml" in result.output


# ---------------------------------------------------------------------------
# contexter gc
# ---------------------------------------------------------------------------


class TestGc:
    """Tests for `contexter gc`."""

    @patch("contexter_server.cli.status_commands.StorageEngine")
    def test_gc_flush_and_checkpoint(self, mock_storage: AsyncMock, runner: CliRunner) -> None:
        engine_instance = mock_storage.return_value
        engine_instance.flush = AsyncMock(return_value=None)
        engine_instance.checkpoint = AsyncMock(return_value=42)

        result = runner.invoke(cli, ["gc"])

        assert result.exit_code == 0, f"Output: {result.output}"
        assert "Garbage collection complete" in result.output
        assert "42" in result.output
        engine_instance.flush.assert_awaited_once()
        engine_instance.checkpoint.assert_awaited_once()

    @patch("contexter_server.cli.status_commands.StorageEngine")
    def test_gc_handles_error(self, mock_storage: AsyncMock, runner: CliRunner) -> None:
        engine_instance = mock_storage.return_value
        engine_instance.flush = AsyncMock(side_effect=RuntimeError("Engine error"))

        result = runner.invoke(cli, ["gc"])

        assert result.exit_code != 0
        # The underlying error is reported through the structured log
        # (caplog), while the user-facing message comes from ClickException.
        assert "Garbage collection failed" in result.output


# ---------------------------------------------------------------------------
# contexter --engine-path
# ---------------------------------------------------------------------------


class TestEnginePath:
    """Tests for the global --engine-path option."""

    @patch("contexter_server.cli.status_commands.StorageEngine")
    def test_engine_path_option(self, mock_storage: AsyncMock, runner: CliRunner) -> None:
        engine_instance = mock_storage.return_value
        engine_instance.flush = AsyncMock(return_value=None)
        engine_instance.checkpoint = AsyncMock(return_value=1)

        result = runner.invoke(cli, ["--engine-path", "/tmp/test-contexter", "gc"])

        assert result.exit_code == 0
        mock_storage.assert_called_once_with("/tmp/test-contexter")
