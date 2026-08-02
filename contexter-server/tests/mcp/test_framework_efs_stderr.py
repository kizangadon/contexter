"""Framework-level EFS regression tests (REQ-FL-001..005, EC-FL-001..007).

Bug contract: 2026-08-01-fastmcp-framework-logging.

The iter-2 gap (EC-FL-007) was that ``test_bridge_engine_failure_stderr.py``
(13 tests) covered the bridge stdlib logger scope only — never the FastMCP
framework logger (``fastmcp.*``, ``propagate=False``), whose generic
``except Exception`` path (``fastmcp/server/server.py:1297``) runs
``logger.exception`` and renders a 2672-char rich traceback box on stderr
for every tool error.

These tests run tool/resource errors through the REAL FastMCP call path
(``create_mcp_server`` -> FastMCP wrapper -> handler -> real service -> real
engine) and assert the framework's stderr contribution stays bounded
(<=512 chars, no rich box, no raw traceback) for every error class, while
client-visible ``isError`` frames stay byte-identical to the pre-fix
baseline (captured with ``probe_baseline_frames.py``, REQ-FL-002).

Stderr observation model (REQ-EP-002)
-------------------------------------
In-process ``capfd`` measures the FRAMEWORK contribution only.  The
bridge's structlog ERROR records (``contexter_server.core.bridge``,
propagate=True) are captured by pytest's root ``LogCaptureHandler``
(installed around every test) and never reach fd-2; ``logging.lastResort``
never fires.  The FastMCP framework logger (``fastmcp.*``,
propagate=False) writes through its own StreamHandler to fd-2, so its
output — the startup banner, warnings and, absent the filter, the
2672-char rich traceback box — lands in the capfd buffer.  The <=512-byte
assertions therefore bound the framework's per-failure contribution and
stay discriminating because the rich box travels exactly this path.

The end-to-end budget (bridge line + framework output on real fd-2) is
covered LIVE by the subprocess evidence in
``tests/core/test_bridge_live_coverage.py``
(``TestLiveFailureStderrEvidence``) and was independently verified by the
user-testing validator (auth 186B, not_found 294B, engine 326B).
"""

import asyncio

import pytest

from mcp.shared.exceptions import McpError

from contexter_server.core.bridge import StorageEngine
from contexter_server.mcp_server import create_mcp_server
from contexter_server.services.agent_service import AgentService
from contexter_server.services.analytics_service import AnalyticsService
from contexter_server.services.memory_service import MemoryService
from contexter_server.services.session_service import SessionService
from contexter_server.services.skill_service import SkillService

_SID = "00000000-0000-0000-0000-000000000001"
_MISSING_ID = "deadbeef-0000-0000-0000-000000000000"
_INVALID_ID = "not-a-uuid"
_TEST_KEY = "test-key-123"

_STDERR_LIMIT = 512

# Rich box drawing characters (REQ-FL-001 / AC-FL-001).
_BOX_CHARS = ("╭", "│", "╰")

# Pre-fix client-visible isError frames, captured live via
# /tmp/opencode/probe_baseline_frames.py (REQ-FL-002).  These MUST stay
# byte-identical after the stderr-hygiene fix.
BASELINE_FRAMES = {
    # Engine message pinned from the live engine: the Rust engine formats
    # the offending value with double quotes (probe + direct engine call).
    "engine": (
        "Error calling tool 'get_session': invalid session id \"not-a-uuid\": "
        "invalid character: found `n` at 0"
    ),
    "not_found": (
        f"Error calling tool 'get_session': Resource not found: {_MISSING_ID}"
    ),
    "storage": "Error calling tool 'store_memory': MCP server not connected to storage",
    "auth_missing": (
        "Error calling tool 'get_session': API key required. Provide a matching "
        "_api_key parameter or unset CONTEXTER_API_KEY to disable authentication."
    ),
    "auth_wrong": "Error calling tool 'get_session': Invalid API key.",
    "validation_empty": "Error calling tool 'store_memory': content must not be empty",
    "validation_query": (
        "Error calling tool 'search_memories': query exceeds maximum length of 10000"
    ),
}


def _assert_bounded(stderr: str, label: str) -> None:
    """Assert the total stderr for one failure is <=512 chars, no box/traceback."""
    assert len(stderr) <= _STDERR_LIMIT, (
        f"{label}: {len(stderr)} chars > {_STDERR_LIMIT}: {stderr!r}"
    )
    assert len(stderr.encode("utf-8")) <= _STDERR_LIMIT, (
        f"{label}: {len(stderr.encode('utf-8'))} bytes > {_STDERR_LIMIT}"
    )
    for ch in _BOX_CHARS:
        assert ch not in stderr, f"{label}: rich box char {ch!r} present"
    assert "Traceback" not in stderr, f"{label}: raw Traceback present"
    assert 'File "' not in stderr, f"{label}: source frame present"


@pytest.fixture
def diag_env(tmp_path, monkeypatch):
    """Pin the diagnostics log to the test dir and start with no API key."""
    log_path = tmp_path / "mcp-launch.log"
    monkeypatch.setenv("CONTEXTER_LOG_FILE", str(log_path))
    monkeypatch.delenv("CONTEXTER_API_KEY", raising=False)
    return str(log_path)


@pytest.fixture
def make_server(tmp_path):
    """Factory: real FastMCP server over a real engine + real services."""
    servers = []

    def _make(with_memory: bool = True):
        engine = StorageEngine(str(tmp_path / f"engine-{len(servers)}"))
        services = {
            "memory_service": MemoryService(engine) if with_memory else None,
            "session_service": SessionService(engine) if with_memory else None,
            "agent_service": AgentService(engine),
            "skill_service": SkillService(engine),
            "analytics_service": AnalyticsService(engine),
            "export_service": None,
        }
        mcp = create_mcp_server(**services)
        assert mcp is not None
        servers.append(engine)
        return mcp

    yield _make
    for engine in servers:
        engine._pool.shutdown(wait=True)


class TestFrameworkStderrBounded:
    """Every error class through the live FastMCP path: stderr <=512, no box."""

    @pytest.mark.asyncio
    async def test_engine_failure_stderr_bounded_frame_stable(
        self, diag_env, make_server, capfd
    ):
        """Invalid session id -> bridge ValueError: bounded stderr, stable frame."""
        mcp = make_server()

        from fastmcp import Client

        async with Client(mcp) as client:
            capfd.readouterr()  # drain startup output
            result = await client.call_tool_mcp("get_session", {"id": _INVALID_ID})

        assert result.isError is True
        assert result.content[0].text == BASELINE_FRAMES["engine"]
        _assert_bounded(capfd.readouterr().err, "engine failure")

    @pytest.mark.asyncio
    async def test_not_found_stderr_bounded_frame_stable(
        self, diag_env, make_server, capfd
    ):
        """Unknown session id -> structured not-found: bounded stderr, stable frame."""
        mcp = make_server()

        from fastmcp import Client

        async with Client(mcp) as client:
            capfd.readouterr()
            result = await client.call_tool_mcp("get_session", {"id": _MISSING_ID})

        assert result.isError is True
        assert result.content[0].text == BASELINE_FRAMES["not_found"]
        _assert_bounded(capfd.readouterr().err, "not-found failure")

    @pytest.mark.asyncio
    async def test_storage_not_connected_stderr_bounded_frame_stable(
        self, diag_env, make_server, capfd
    ):
        """Server without storage services -> storage error: bounded, stable frame."""
        mcp = make_server(with_memory=False)

        from fastmcp import Client

        async with Client(mcp) as client:
            capfd.readouterr()
            result = await client.call_tool_mcp(
                "store_memory",
                {"session_id": _SID, "role": "user", "content": "hello"},
            )

        assert result.isError is True
        assert result.content[0].text == BASELINE_FRAMES["storage"]
        _assert_bounded(capfd.readouterr().err, "storage failure")

    @pytest.mark.asyncio
    async def test_auth_missing_stderr_bounded_frame_stable(
        self, diag_env, make_server, monkeypatch, capfd
    ):
        """Key configured, no _api_key -> MCPAuthError: bounded, stable frame."""
        monkeypatch.setenv("CONTEXTER_API_KEY", _TEST_KEY)
        mcp = make_server()

        from fastmcp import Client

        async with Client(mcp) as client:
            capfd.readouterr()
            result = await client.call_tool_mcp("get_session", {"id": _SID})

        assert result.isError is True
        assert result.content[0].text == BASELINE_FRAMES["auth_missing"]
        _assert_bounded(capfd.readouterr().err, "auth missing-key failure")

    @pytest.mark.asyncio
    async def test_auth_wrong_stderr_bounded_frame_stable(
        self, diag_env, make_server, monkeypatch, capfd
    ):
        """Key configured, wrong _api_key -> MCPAuthError: bounded, stable frame."""
        monkeypatch.setenv("CONTEXTER_API_KEY", _TEST_KEY)
        mcp = make_server()

        from fastmcp import Client

        async with Client(mcp) as client:
            capfd.readouterr()
            result = await client.call_tool_mcp(
                "get_session", {"id": _SID, "_api_key": "wrong-key"}
            )

        assert result.isError is True
        assert result.content[0].text == BASELINE_FRAMES["auth_wrong"]
        _assert_bounded(capfd.readouterr().err, "auth wrong-key failure")

    @pytest.mark.asyncio
    async def test_validation_empty_content_stderr_bounded_frame_stable(
        self, diag_env, make_server, capfd
    ):
        """Empty content -> validation error: bounded stderr, stable frame."""
        mcp = make_server()

        from fastmcp import Client

        async with Client(mcp) as client:
            capfd.readouterr()
            result = await client.call_tool_mcp(
                "store_memory",
                {"session_id": _SID, "role": "user", "content": ""},
            )

        assert result.isError is True
        assert result.content[0].text == BASELINE_FRAMES["validation_empty"]
        _assert_bounded(capfd.readouterr().err, "validation failure")

    @pytest.mark.asyncio
    async def test_oversized_query_no_content_leak_stderr_bounded(
        self, diag_env, make_server, capfd
    ):
        """10KB query -> validation error; content must never reach stderr (EC-FL-004)."""
        oversized = "q" * (10_000 + 1)
        mcp = make_server()

        from fastmcp import Client

        async with Client(mcp) as client:
            capfd.readouterr()
            result = await client.call_tool_mcp("search_memories", {"query": oversized})

        assert result.isError is True
        assert result.content[0].text == BASELINE_FRAMES["validation_query"]
        stderr = capfd.readouterr().err
        _assert_bounded(stderr, "oversized-query failure")
        assert oversized not in stderr, "unbounded input leaked to stderr (EC-FL-004)"

    @pytest.mark.asyncio
    async def test_resource_read_error_stderr_bounded(
        self, diag_env, make_server, capfd
    ):
        """Resource read failure: bounded stderr, no box (REQ-FL-001 resource path)."""
        mcp = make_server()

        from fastmcp import Client

        async with Client(mcp) as client:
            capfd.readouterr()
            with pytest.raises(McpError):
                await client.read_resource(f"contexter://session/{_MISSING_ID}")

        _assert_bounded(capfd.readouterr().err, "resource read failure")

    @pytest.mark.asyncio
    async def test_concurrent_failures_each_bounded(self, diag_env, make_server, capfd):
        """Concurrent failures: no combined giant stderr block (EC-FL-005)."""
        mcp = make_server()
        n = 5

        from fastmcp import Client

        async with Client(mcp) as client:
            capfd.readouterr()
            results = await asyncio.gather(
                *(
                    client.call_tool_mcp("get_session", {"id": _INVALID_ID})
                    for _ in range(n)
                )
            )

        for result in results:
            assert result.isError is True
            assert result.content[0].text == BASELINE_FRAMES["engine"]
        stderr = capfd.readouterr().err
        # EC-FL-005: the single-failure budget applies to the COMBINED block —
        # without the filter, n concurrent failures would each emit a rich box
        # (~2672B), blowing the <=512 budget; a looser n * _STDERR_LIMIT bound
        # would not discriminate (F-3), so it is deliberately not asserted.
        _assert_bounded(stderr, "concurrent failures")


class TestFrameworkSuccessAndDiagnostics:
    """Success path and diagnostics channel stay untouched (REQ-FL-003/004)."""

    @pytest.mark.asyncio
    async def test_success_path_stderr_no_new_noise(self, diag_env, make_server, capfd):
        """Successful call: no error records, no box, no traceback (AC-FL-005)."""
        mcp = make_server()

        from fastmcp import Client

        async with Client(mcp) as client:
            capfd.readouterr()
            result = await client.call_tool_mcp("list_recent_sessions", {})

        assert result.isError is False
        stderr = capfd.readouterr().err
        assert "Error calling tool" not in stderr
        assert "Error reading resource" not in stderr
        assert "Traceback" not in stderr
        for ch in _BOX_CHARS:
            assert ch not in stderr

    @pytest.mark.asyncio
    async def test_diagnostics_log_retains_full_traceback(
        self, diag_env, make_server, capfd
    ):
        """Engine failure: full traceback still in CONTEXTER_LOG_FILE (AC-FL-003)."""
        mcp = make_server()

        from fastmcp import Client

        async with Client(mcp) as client:
            capfd.readouterr()
            await client.call_tool_mcp("get_session", {"id": _INVALID_ID})

        from pathlib import Path

        diag_log = Path(diag_env)
        assert diag_log.exists(), "diagnostics log not written"
        content = diag_log.read_text()
        assert "Traceback" in content, "diagnostics log lost the traceback"
        assert "invalid session id" in content


class TestFrameworkConfigDirectNamespace:
    """EC-FL-001: the fix must target the fastmcp namespace directly."""

    def test_fastmcp_logger_has_bounded_stderr_filter(self):
        """logging.getLogger('fastmcp') carries the suppression filter."""
        import logging

        from contexter_server.fastmcp_logging import _SuppressFrameworkTracebackBox

        logger = logging.getLogger("fastmcp")
        assert any(
            isinstance(f, _SuppressFrameworkTracebackBox) for f in logger.filters
        ), "fastmcp logger is missing the bounded-stderr filter (propagate=False!)"

    def test_filter_drops_framework_error_records(self):
        """Unit: error-call records are dropped; other records pass through."""
        import logging

        from contexter_server.fastmcp_logging import _SuppressFrameworkTracebackBox

        filt = _SuppressFrameworkTracebackBox()

        for prefix in (
            "Error calling tool",
            "Error reading resource",
            "Error rendering prompt",
        ):
            for exc_info in ((ValueError, ValueError("x"), None), None):
                record = logging.LogRecord(
                    "fastmcp.server.server",
                    logging.ERROR,
                    "server.py",
                    1297,
                    f"{prefix} 'get_session'",
                    (),
                    exc_info,
                )
                assert filt.filter(record) is False, f"{prefix} must be dropped"

        record = logging.LogRecord(
            "fastmcp.server.server",
            logging.WARNING,
            "server.py",
            1290,
            "Invalid arguments for tool %r: %s",
            ("get_session", "[]"),
            None,
        )
        # REQ-FC-002/REQ-FC-005 (bug contract 2026-08-01-fastmcp-filter-coverage):
        # the schema-validation WARNING carries exc_info + a file:line reference
        # and MUST be dropped at every level, not passed through.
        assert filt.filter(record) is False

        record = logging.LogRecord(
            "fastmcp.server",
            logging.INFO,
            "server.py",
            100,
            "Registered %d tools",
            (3,),
            None,
        )
        assert filt.filter(record) is True
