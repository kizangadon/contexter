"""Protocol-level edge case regression tests for the parent MCP contract.

Covers parent EDGE_CASES.md rows (bug contract ``2026-08-01-parent-edge-case-tests``):

- EC-015  Wrong JSON-RPC payload from client
          -> Protocol-level error response; process alive                    (P2)
- EC-017  Two concurrent tool calls (parallel stdio requests)
          -> Both complete; no cross-talk; no interleaved stdout corruption  (P2)
- EC-018  Concurrent ``store_memory`` to same session
          -> Both persist; engine serializes via bridge thread pool          (P3)
- EC-021  Client disconnects mid-call
          -> Server handles cleanly; no zombie process                       (P3)

Mapping (EC-PEC-001): EC-015 and EC-021 were previously verified only manually
(user-testing review 2026-08-02); EC-017/EC-018 had no coverage at all.  No
pre-existing automated test duplicates any of the four scenarios: the existing
concurrency tests (``tests/core/test_bridge.py``, ``tests/services/test_edge_cases.py``)
exercise the engine thread pool and service layers, never the MCP protocol.

Implementation-vs-docs check (REQ-PEC-003): every documented behavior here was
probed against the live implementation before writing these tests and the
implementation matches the documentation — this file adds tests only.

Subprocess tests spawn the real ``run_mcp.py`` stdio server and speak raw
JSON-RPC over the wire.  In-process tests use the live FastMCP client over a
real ``StorageEngine`` (RocksDB in ``tmp_path``) with real domain services.
All waiting uses bounded select-based deadlines, never sleeps.
"""

import asyncio
import json
import os
import select
import subprocess
import sys
import time
import uuid
from pathlib import Path

import pytest
from fastmcp import Client

from contexter_server.core.bridge import StorageEngine
from contexter_server.mcp_server import create_mcp_server
from contexter_server.models.memory import MemoryCreate
from contexter_server.models.search import SearchResponse, SearchResult
from contexter_server.models.session import SessionCreate
from contexter_server.services.agent_service import AgentService
from contexter_server.services.analytics_service import AnalyticsService
from contexter_server.services.memory_service import MemoryService
from contexter_server.services.session_service import SessionService
from contexter_server.services.skill_service import SkillService

_REPO_ROOT = Path(__file__).resolve().parents[2]
_AGENT_ID = uuid.UUID("00000000-0000-0000-0000-000000000001")
_INIT_ID = 1
_INIT_REQUEST = {
    "jsonrpc": "2.0",
    "id": _INIT_ID,
    "method": "initialize",
    "params": {
        "protocolVersion": "2025-03-26",
        "capabilities": {},
        "clientInfo": {"name": "protocol-edge-tests", "version": "1.0"},
    },
}


class _StdioServer:
    """A real ``run_mcp.py`` stdio MCP server child process.

    Spawns the actual launcher with a private data dir and launch log, talks
    raw JSON-RPC lines over the pipes, and fails tests with diagnostics when
    the child exits early, closes stdout, or emits a non-JSON line.
    """

    def __init__(self, tmp_path):
        env = os.environ.copy()
        env["CONTEXTER_PATH"] = str(tmp_path / "data")
        env["CONTEXTER_LOG_FILE"] = str(tmp_path / "launch.log")
        # Deterministic open-mode auth regardless of the outer environment.
        env.pop("CONTEXTER_API_KEY", None)
        self.proc = subprocess.Popen(
            [sys.executable, "run_mcp.py"],
            cwd=str(_REPO_ROOT),
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            env=env,
        )
        self._stderr_tail = ""

    def send(self, payload) -> None:
        """Write one line to the server's stdin (dict -> JSON-RPC frame)."""
        line = json.dumps(payload) if not isinstance(payload, str) else payload
        self.proc.stdin.write(line + "\n")
        self.proc.stdin.flush()

    def initialize(self) -> None:
        """Perform the mandatory MCP ``initialize`` handshake.

        FastMCP rejects any request that arrives before initialization with
        -32602 (MCP spec: a client MUST initialize first), so every raw-wire
        test performs the handshake before exercising the documented behavior.
        """
        self.send(_INIT_REQUEST)
        resp = self.read_until(lambda m: m.get("id") == _INIT_ID)
        assert "result" in resp, f"initialize handshake failed: {resp!r}"

    def read_until(self, predicate, timeout: float = 15.0) -> dict:
        """Read JSON-RPC messages until ``predicate`` matches one.

        Every consumed line MUST parse as complete JSON (frame integrity);
        a non-JSON line on stdout is a protocol violation and fails the test.
        The deadline is a bounded liveness guard, not a timing assertion.
        """
        buf = ""
        deadline = time.monotonic() + timeout
        while True:
            if self.proc.poll() is not None:
                raise AssertionError(
                    f"server exited (code {self.proc.returncode}) before a matching "
                    f"message; stderr: {self._drain_stderr()!r}"
                )
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                raise AssertionError(
                    f"no matching JSON-RPC message within {timeout}s; "
                    f"stderr: {self._drain_stderr()!r}"
                )
            ready, _, _ = select.select(
                [self.proc.stdout], [], [], max(0.0, min(remaining, 1.0))
            )
            if not ready:
                continue
            chunk = os.read(self.proc.stdout.fileno(), 65536).decode(
                "utf-8", "replace"
            )
            if not chunk:
                raise AssertionError(
                    f"server closed stdout; stderr: {self._drain_stderr()!r}"
                )
            buf += chunk
            while "\n" in buf:
                line, buf = buf.split("\n", 1)
                line = line.strip()
                if not line:
                    continue
                try:
                    msg = json.loads(line)
                except json.JSONDecodeError:
                    raise AssertionError(
                        f"non-JSON line on stdout (frame integrity violation): {line!r}"
                    )
                if predicate(msg):
                    return msg

    def _drain_stderr(self) -> str:
        try:
            chunk = os.read(self.proc.stderr.fileno(), 65536).decode(
                "utf-8", "replace"
            )
            if chunk:
                self._stderr_tail += chunk
        except Exception:  # noqa: BLE001 - drain is best-effort diagnostics
            pass
        return self._stderr_tail

    def close_stdin(self) -> None:
        self.proc.stdin.close()

    def wait(self, timeout: float = 15.0) -> int:
        code = self.proc.wait(timeout=timeout)
        self._drain_stderr()
        return code

    def shutdown(self) -> None:
        if self.proc.poll() is None:
            self.proc.kill()
            self.proc.wait()


@pytest.fixture
def live_client(tmp_path):
    """FastMCP client over a real engine + real services on a private RocksDB dir."""
    engine = StorageEngine(str(tmp_path))
    services = {
        "memory_service": MemoryService(engine),
        "session_service": SessionService(engine),
        "agent_service": AgentService(engine),
        "skill_service": SkillService(engine),
        "analytics_service": AnalyticsService(engine),
        "export_service": None,
    }
    mcp = create_mcp_server(**services)
    assert mcp is not None

    client = Client(mcp)
    yield client, services
    engine._pool.shutdown(wait=True)


class TestWrongJsonRpcPayload:
    """EC-015: wrong JSON-RPC payload -> protocol-level error; process alive."""

    def test_malformed_and_unknown_frames_are_protocol_errors_and_server_stays_alive(
        self, tmp_path
    ):
        server = _StdioServer(tmp_path)
        try:
            server.initialize()

            # A non-JSON frame must produce a protocol-level error message.
            server.send("this is not json")
            err = server.read_until(
                lambda m: "error" in m or m.get("method") == "notifications/message"
            )
            if "error" not in err:
                assert err.get("params", {}).get("level") == "error", (
                    f"malformed frame did not yield a protocol error: {err!r}"
                )

            # An unknown method must produce a JSON-RPC error response for its own id.
            server.send(
                {"jsonrpc": "2.0", "id": 7, "method": "bogus/method", "params": {}}
            )
            resp = server.read_until(lambda m: m.get("id") == 7)
            assert "error" in resp, f"unknown method did not yield an error: {resp!r}"
            assert resp["error"]["code"] == -32602, f"unexpected error code: {resp!r}"

            # The server must remain alive and serve valid requests afterwards.
            server.send({"jsonrpc": "2.0", "id": 8, "method": "tools/list"})
            ok = server.read_until(lambda m: m.get("id") == 8)
            assert "result" in ok, f"server not alive after malformed frames: {ok!r}"
            assert server.proc.poll() is None, "server process died"
        finally:
            server.close_stdin()
            server.shutdown()


class TestConcurrentToolCalls:
    """EC-017: concurrent tool calls complete with no cross-talk."""

    @pytest.mark.asyncio
    async def test_concurrent_searches_no_cross_talk(self, live_client):
        """Two parallel searches each answer only their own query (real engine)."""
        client, services = live_client
        async with client:
            session = await services["session_service"].create(
                SessionCreate(agent_id=_AGENT_ID, project="protocol-edge-017")
            )
            await services["memory_service"].create(
                MemoryCreate(
                    session_id=session.id,
                    agent_id=_AGENT_ID,
                    role="user",
                    content="concurrent-alpha-marker",
                )
            )
            await services["memory_service"].create(
                MemoryCreate(
                    session_id=session.id,
                    agent_id=_AGENT_ID,
                    role="user",
                    content="concurrent-beta-marker",
                )
            )

            alpha, beta = await asyncio.wait_for(
                asyncio.gather(
                    client.call_tool(
                        "search_memories", {"query": "concurrent-alpha-marker"}
                    ),
                    client.call_tool(
                        "search_memories", {"query": "concurrent-beta-marker"}
                    ),
                ),
                timeout=15.0,
            )

        assert alpha.is_error is False and beta.is_error is False
        assert '"snippet":"concurrent-alpha-marker"' in alpha.content[0].text
        assert '"snippet":"concurrent-beta-marker"' not in alpha.content[0].text
        assert '"snippet":"concurrent-beta-marker"' in beta.content[0].text
        assert '"snippet":"concurrent-alpha-marker"' not in beta.content[0].text

    @pytest.mark.asyncio
    async def test_concurrent_calls_are_truly_in_flight_simultaneously(self):
        """A gate proves both requests are handled concurrently, not serialized.

        Each search blocks until BOTH handlers have entered, then returns its
        own marker.  A server that serialized requests would deadlock on the
        gate and the call would time out — a deterministic parallelism proof.
        """

        class _GatedSearchService:
            def __init__(self):
                self._entered = asyncio.Event()
                self._count = 0
                self._lock = asyncio.Lock()

            async def search(self, query):
                async with self._lock:
                    self._count += 1
                    if self._count == 2:
                        self._entered.set()
                await self._entered.wait()
                marker = (
                    "alpha-result" if "alpha" in query.query else "beta-result"
                )
                return SearchResponse(
                    results=[
                        SearchResult(
                            id="00000000-0000-0000-0000-0000000000a1",
                            type="memory",
                            score=0.9,
                            snippet=marker,
                        )
                    ],
                    total=1,
                )

        gated = _GatedSearchService()
        mcp = create_mcp_server(memory_service=gated)
        assert mcp is not None

        async with Client(mcp) as client:
            alpha, beta = await asyncio.wait_for(
                asyncio.gather(
                    client.call_tool("search_memories", {"query": "alpha"}),
                    client.call_tool("search_memories", {"query": "beta"}),
                ),
                timeout=10.0,
            )

        assert alpha.is_error is False and beta.is_error is False
        assert '"snippet":"alpha-result"' in alpha.content[0].text
        assert '"snippet":"beta-result"' not in alpha.content[0].text
        assert '"snippet":"beta-result"' in beta.content[0].text
        assert '"snippet":"alpha-result"' not in beta.content[0].text

    def test_parallel_stdio_requests_produce_intact_frames(self, tmp_path):
        """Two parallel stdio requests are both answered with intact, matching frames."""
        server = _StdioServer(tmp_path)
        try:
            server.initialize()
            server.send({"jsonrpc": "2.0", "id": 11, "method": "tools/list"})
            server.send({"jsonrpc": "2.0", "id": 12, "method": "tools/list"})
            m1 = server.read_until(lambda m: m.get("id") in (11, 12))
            m2 = server.read_until(lambda m: m.get("id") in (11, 12))
            # Frame integrity is enforced inside read_until (each line must
            # parse as complete JSON); here we verify both requests were
            # answered exactly once with their own result.
            assert {m1["id"], m2["id"]} == {11, 12}
            assert "result" in m1, f"frame missing result: {m1!r}"
            assert "result" in m2, f"frame missing result: {m2!r}"
            assert server.proc.poll() is None, "server process died"
        finally:
            server.close_stdin()
            server.shutdown()


class TestConcurrentStoreMemorySameSession:
    """EC-018: concurrent ``store_memory`` to the same session — both persist."""

    @pytest.mark.asyncio
    async def test_concurrent_store_memory_same_session_both_persist(self, live_client):
        client, services = live_client
        async with client:
            session = await services["session_service"].create(
                SessionCreate(agent_id=_AGENT_ID, project="protocol-edge-018")
            )
            sid = str(session.id)

            alpha, beta = await asyncio.wait_for(
                asyncio.gather(
                    client.call_tool(
                        "store_memory",
                        {
                            "session_id": sid,
                            "role": "user",
                            "content": "alpha-marker",
                        },
                    ),
                    client.call_tool(
                        "store_memory",
                        {
                            "session_id": sid,
                            "role": "user",
                            "content": "beta-marker",
                        },
                    ),
                ),
                timeout=15.0,
            )

            assert alpha.is_error is False and beta.is_error is False
            payload_a = json.loads(alpha.content[0].text)
            payload_b = json.loads(beta.content[0].text)
            assert payload_a["memory_id"] != payload_b["memory_id"], (
                "concurrent stores must create distinct memories"
            )

            found_a = await client.call_tool(
                "search_memories", {"query": "alpha-marker"}
            )
            found_b = await client.call_tool(
                "search_memories", {"query": "beta-marker"}
            )

        assert '"snippet":"alpha-marker"' in found_a.content[0].text
        assert '"snippet":"beta-marker"' in found_b.content[0].text


class TestClientDisconnect:
    """EC-021: client disconnects mid-call — clean handling, no zombie."""

    def test_disconnect_mid_call_exits_cleanly_no_zombie(self, tmp_path):
        server = _StdioServer(tmp_path)
        # Establish the handshake, send a call, then close stdin before
        # reading the response: the call is in flight when the client
        # disconnects.
        server.initialize()
        server.send({"jsonrpc": "2.0", "id": 5, "method": "tools/list"})
        server.close_stdin()

        code = server.wait(timeout=15.0)
        assert code == 0, (
            f"server must exit cleanly on client disconnect, got {code}; "
            f"stderr: {server._stderr_tail!r}"
        )
        # wait() reaped the child: a returned exit code means no zombie remains.
        assert server.proc.poll() is not None
