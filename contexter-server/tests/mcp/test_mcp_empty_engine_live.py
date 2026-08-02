"""Regression tests: MCP tools behave gracefully against an EMPTY engine (AC-7).

AC-7 (parent SPEC): given a running MCP server connected to an empty engine
(no sessions, no memories, no agents, no skills), list/overview calls return
empty results with success status — never errors.

These tests build a REAL ``StorageEngine`` over a private RocksDB dir
(``tmp_path``) and wire the REAL domain services, so the empty-engine path
is exercised through the live protocol: ``create_mcp_server()`` → FastMCP
tool wrapper → handler → real service → real Rust engine.  Every call uses
the in-process FastMCP client (the live protocol path), matching the
sibling live tests.
"""

import pytest

from contexter_server.core.bridge import StorageEngine
from contexter_server.mcp_server import create_mcp_server
from contexter_server.services.agent_service import AgentService
from contexter_server.services.analytics_service import AnalyticsService
from contexter_server.services.memory_service import MemoryService
from contexter_server.services.session_service import SessionService
from contexter_server.services.skill_service import SkillService

_SID = "00000000-0000-0000-0000-000000000001"


@pytest.fixture
def empty_engine_client(tmp_path):
    """FastMCP client over a real engine + real services on an empty RocksDB dir."""
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

    from fastmcp import Client

    client = Client(mcp)
    yield client, services
    # Note: the client context is entered/exited per-test to keep each test
    # self-contained; the engine pool is shut down here.
    engine._pool.shutdown(wait=True)


class TestEmptyEngineLists:
    """List tools on an empty engine return empty results with success (AC-7)."""

    @pytest.mark.asyncio
    async def test_list_recent_sessions_empty(self, empty_engine_client):
        client, _ = empty_engine_client
        async with client:
            result = await client.call_tool("list_recent_sessions", {})
        assert result.is_error is False
        assert '"sessions":[]' in result.content[0].text

    @pytest.mark.asyncio
    async def test_search_memories_empty(self, empty_engine_client):
        client, _ = empty_engine_client
        async with client:
            result = await client.call_tool("search_memories", {"query": "x"})
        assert result.is_error is False
        assert '"results":[]' in result.content[0].text
        assert '"total":0' in result.content[0].text

    @pytest.mark.asyncio
    async def test_list_skills_empty(self, empty_engine_client):
        client, _ = empty_engine_client
        async with client:
            result = await client.call_tool("list_skills", {})
        assert result.is_error is False
        assert '"skills":[]' in result.content[0].text


class TestEmptyEngineOverview:
    """Overview/health tools on an empty engine return success (AC-7)."""

    @pytest.mark.asyncio
    async def test_get_system_health_empty(self, empty_engine_client):
        client, _ = empty_engine_client
        async with client:
            result = await client.call_tool("get_system_health", {})
        assert result.is_error is False
        assert '"status":"ok"' in result.content[0].text


class TestEmptyEngineNotFoundShapes:
    """Point reads on an empty engine produce structured not-found errors,
    never crashes or raw engine exceptions."""

    @pytest.mark.asyncio
    async def test_get_session_unknown_id_is_structured_error(self, empty_engine_client):
        from fastmcp.exceptions import ToolError

        client, _ = empty_engine_client
        async with client:
            with pytest.raises(ToolError, match="Resource not found"):
                await client.call_tool("get_session", {"id": _SID})

    @pytest.mark.asyncio
    async def test_store_memory_without_session_is_structured_error(self, empty_engine_client):
        from fastmcp.exceptions import ToolError

        client, _ = empty_engine_client
        async with client:
            with pytest.raises(ToolError, match="Resource not found"):
                await client.call_tool(
                    "store_memory",
                    {"session_id": _SID, "role": "user", "content": "hello"},
                )
