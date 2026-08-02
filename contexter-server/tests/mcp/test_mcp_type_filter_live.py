"""Regression tests: ``list_skills`` / ``search_memories`` through the FastMCP wrapper.

These tests exercise the full registration path — ``create_mcp_server()`` →
FastMCP tool wrapper → handler → service — through an in-process FastMCP
client (``fastmcp.Client(mcp)``), matching the live MCP protocol path.

They guard the schema/handler drift defect (SPEC AC-003, EC-004): the wrapper
forwards ``type=type`` (the frozen contract parameter name) while the handlers
were renamed to ``type_filter``, so every live call failed with
``TypeError: ... got an unexpected keyword argument 'type'``. The previous
handler-level tests passed for the wrong reason because they called the
handlers directly and never went through the wrapper.
"""

from unittest.mock import AsyncMock
from uuid import UUID

import pytest

from contexter_server.mcp_server import create_mcp_server
from contexter_server.models.search import SearchResponse, SearchResult
from contexter_server.models.skill import Skill


@pytest.fixture
def live_server():
    """Build a real FastMCP server with AsyncMock services and return (mcp, services)."""
    services = {
        "memory_service": AsyncMock(),
        "session_service": AsyncMock(),
        "agent_service": AsyncMock(),
        "skill_service": AsyncMock(),
        "analytics_service": AsyncMock(),
        "export_service": AsyncMock(),
    }
    mcp = create_mcp_server(**services)
    assert mcp is not None
    return mcp, services


class TestListSkillsLive:
    """list_skills through the wrapper must accept the registered ``type`` parameter."""

    @pytest.mark.asyncio
    async def test_list_skills_with_type_returns_filtered_data(self, live_server):
        """AC-003: client.call_tool('list_skills', {'type': 'mcp'}) succeeds and filters."""
        mcp, services = live_server
        skill = Skill(
            id=UUID("00000000-0000-0000-0000-000000000001"),
            name="mcp-skill",
            type="mcp",
        )
        services["skill_service"].list.return_value = [skill]

        from fastmcp import Client

        async with Client(mcp) as client:
            result = await client.call_tool("list_skills", {"type": "mcp"})

        assert result.is_error is False
        assert "mcp-skill" in result.content[0].text
        services["skill_service"].list.assert_awaited_once_with(filter={"type": "mcp"})

    @pytest.mark.asyncio
    async def test_list_skills_without_type_succeeds(self, live_server):
        """AC-003: list_skills with type omitted succeeds with no filter (empty-filter)."""
        mcp, services = live_server
        services["skill_service"].list.return_value = []

        from fastmcp import Client

        async with Client(mcp) as client:
            result = await client.call_tool("list_skills", {})

        assert result.is_error is False
        services["skill_service"].list.assert_awaited_once_with(filter=None)


class TestSearchMemoriesLive:
    """search_memories through the wrapper must accept the registered ``type`` parameter."""

    @pytest.mark.asyncio
    async def test_search_memories_with_type_returns_filtered_data(self, live_server):
        """AC-003: client.call_tool('search_memories', {'query': 'x', 'type': 'memory'}) succeeds."""
        mcp, services = live_server
        services["memory_service"].search.return_value = SearchResponse(
            results=[
                SearchResult(
                    id=UUID("00000000-0000-0000-0000-000000000002"),
                    type="memory",
                    score=0.95,
                    snippet="probe content",
                ),
            ],
            total=1,
        )

        from fastmcp import Client

        async with Client(mcp) as client:
            result = await client.call_tool(
                "search_memories", {"query": "x", "type": "memory"}
            )

        assert result.is_error is False
        assert "probe content" in result.content[0].text
        search_query = services["memory_service"].search.call_args[0][0]
        assert search_query.type == "memory"
        assert search_query.query == "x"

    @pytest.mark.asyncio
    async def test_search_memories_without_type_succeeds(self, live_server):
        """AC-003: search_memories with type omitted succeeds (type=None query)."""
        mcp, services = live_server
        services["memory_service"].search.return_value = SearchResponse()

        from fastmcp import Client

        async with Client(mcp) as client:
            result = await client.call_tool("search_memories", {"query": "x"})

        assert result.is_error is False
        search_query = services["memory_service"].search.call_args[0][0]
        assert search_query.type is None


class TestRegisteredSchema:
    """The registered input schemas must keep advertising ``type`` (frozen contract)."""

    @pytest.mark.asyncio
    async def test_schema_declares_type_not_type_filter(self, live_server):
        """SPEC 4.1: list_skills/search_memories schemas declare ``type``, never ``type_filter``."""
        mcp, _ = live_server
        tools = await mcp.list_tools()
        by_name = {tool.name: tool for tool in tools}

        for tool_name in ("list_skills", "search_memories"):
            tool = by_name[tool_name]
            schema = tool.input_schema if hasattr(tool, "input_schema") else tool.parameters
            properties = schema.get("properties", {})
            assert "type" in properties, (
                f"{tool_name} schema must advertise 'type' (got: {sorted(properties)})"
            )
            assert "type_filter" not in properties, (
                f"{tool_name} schema must not advertise 'type_filter'"
            )
