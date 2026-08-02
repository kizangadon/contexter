"""Regression tests: MCP resource reads authenticate when CONTEXTER_API_KEY is set.

Investigation T3 found: when ``CONTEXTER_API_KEY`` is set, reads of
``contexter://session/{id}``, ``contexter://memory/{id}`` and
``contexter://agent/{id}`` were **permanently rejected** — the URI templates
had no ``{?_api_key}`` query slot, so there was no transport path for the key
(only ``contexter://analytics/overview{?_api_key}`` had one).

The fix adds the ``{?_api_key}`` RFC 6570 query block to the three resource
templates (preserving BUG-029 design). These tests prove the correct key
succeeds and a missing/wrong key is rejected, through the in-process FastMCP
client (the live protocol path).
"""

import os
from unittest import mock
from unittest.mock import AsyncMock
from uuid import UUID

import pytest

from mcp.shared.exceptions import McpError

from contexter_server.mcp_server import create_mcp_server
from contexter_server.models.agent import Agent
from contexter_server.models.memory import Memory
from contexter_server.models.session import Session

TEST_KEY = "test-key-123"
SID = "00000000-0000-0000-0000-000000000001"


@pytest.fixture
def auth_env():
    """Set the canonical CONTEXTER_API_KEY (all other env vars cleared)."""
    with mock.patch.dict(os.environ, {"CONTEXTER_API_KEY": TEST_KEY}, clear=True):
        yield


@pytest.fixture
def resource_server():
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


class TestSessionResourceAuthLive:
    """contexter://session/{id}{?_api_key} authentication."""

    @pytest.mark.asyncio
    async def test_read_with_correct_key_succeeds(self, auth_env, resource_server):
        """Correct _api_key query param must authenticate the session resource read."""
        mcp, services = resource_server
        services["session_service"].get.return_value = Session(
            id=UUID(SID),
            agent_id=UUID(SID),
            project="test-project",
            name="Test Session",
        )

        from fastmcp import Client

        async with Client(mcp) as client:
            contents = await client.read_resource(
                f"contexter://session/{SID}?_api_key={TEST_KEY}"
            )

        assert "Test Session" in contents[0].text

    @pytest.mark.asyncio
    async def test_read_without_key_rejected(self, auth_env, resource_server):
        """Missing _api_key must be rejected when a key is configured."""
        mcp, services = resource_server
        services["session_service"].get.return_value = Session(
            id=UUID(SID),
            agent_id=UUID(SID),
            project="test-project",
            name="Test Session",
        )

        from fastmcp import Client

        async with Client(mcp) as client:
            with pytest.raises(McpError, match="API key required"):
                await client.read_resource(f"contexter://session/{SID}")

    @pytest.mark.asyncio
    async def test_read_with_wrong_key_rejected(self, auth_env, resource_server):
        """Wrong _api_key must be rejected when a key is configured."""
        mcp, services = resource_server
        services["session_service"].get.return_value = Session(
            id=UUID(SID),
            agent_id=UUID(SID),
            project="test-project",
            name="Test Session",
        )

        from fastmcp import Client

        async with Client(mcp) as client:
            with pytest.raises(McpError, match="Invalid API key"):
                await client.read_resource(
                    f"contexter://session/{SID}?_api_key=wrong-key"
                )


class TestMemoryResourceAuthLive:
    """contexter://memory/{id}{?_api_key} authentication."""

    @pytest.mark.asyncio
    async def test_read_with_correct_key_succeeds(self, auth_env, resource_server):
        """Correct _api_key query param must authenticate the memory resource read."""
        mcp, services = resource_server
        services["memory_service"].get.return_value = Memory(
            id=UUID(SID),
            session_id=UUID(SID),
            agent_id=UUID(SID),
            role="user",
            content="Hello, world!",
        )

        from fastmcp import Client

        async with Client(mcp) as client:
            contents = await client.read_resource(
                f"contexter://memory/{SID}?_api_key={TEST_KEY}"
            )

        assert "Hello, world!" in contents[0].text

    @pytest.mark.asyncio
    async def test_read_without_key_rejected(self, auth_env, resource_server):
        """Missing _api_key must be rejected when a key is configured."""
        mcp, _ = resource_server

        from fastmcp import Client

        async with Client(mcp) as client:
            with pytest.raises(McpError, match="API key required"):
                await client.read_resource(f"contexter://memory/{SID}")


class TestAgentResourceAuthLive:
    """contexter://agent/{id}{?_api_key} authentication."""

    @pytest.mark.asyncio
    async def test_read_with_correct_key_succeeds(self, auth_env, resource_server):
        """Correct _api_key query param must authenticate the agent resource read."""
        mcp, services = resource_server
        services["agent_service"].get.return_value = Agent(
            id=UUID(SID),
            name="test-agent",
            provider="openai",
            model="gpt-4",
        )

        from fastmcp import Client

        async with Client(mcp) as client:
            contents = await client.read_resource(
                f"contexter://agent/{SID}?_api_key={TEST_KEY}"
            )

        assert "test-agent" in contents[0].text

    @pytest.mark.asyncio
    async def test_read_without_key_rejected(self, auth_env, resource_server):
        """Missing _api_key must be rejected when a key is configured."""
        mcp, _ = resource_server

        from fastmcp import Client

        async with Client(mcp) as client:
            with pytest.raises(McpError, match="API key required"):
                await client.read_resource(f"contexter://agent/{SID}")


class TestAnalyticsOverviewResourceAuthLive:
    """contexter://analytics/overview{?_api_key} keeps working (regression guard)."""

    @pytest.mark.asyncio
    async def test_read_with_correct_key_succeeds(self, auth_env, resource_server):
        """Correct _api_key query param must authenticate the analytics overview read."""
        from contexter_server.models.analytics import AnalyticsOverview

        mcp, services = resource_server
        services["analytics_service"].get_overview.return_value = AnalyticsOverview(
            total_sessions=10,
            total_memories=100,
            total_agents=3,
            total_skills=5,
            storage_size_bytes=2048000,
            uptime_seconds=7200,
        )

        from fastmcp import Client

        async with Client(mcp) as client:
            contents = await client.read_resource(
                f"contexter://analytics/overview?_api_key={TEST_KEY}"
            )

        assert '"total_sessions": 10' in contents[0].text

    @pytest.mark.asyncio
    async def test_read_without_key_rejected(self, auth_env, resource_server):
        """Missing _api_key must be rejected for the analytics overview when key is set."""
        mcp, _ = resource_server

        from fastmcp import Client

        async with Client(mcp) as client:
            with pytest.raises(McpError, match="API key required"):
                await client.read_resource("contexter://analytics/overview")
