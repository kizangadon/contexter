"""RED reproduction tests — Bug 2026-08-01-error-shape-drift.

The frozen contract (parent REQ-007 / AC-6 / EC-001) requires handler error
paths to surface as *structured MCP errors* (isError=True frames), never as
``{"error": ...}`` success payloads, and not-found entities must use the
``Resource not found: <id>`` message convention.

These tests fail on the unfixed code (handlers return ``{"error": ...}`` as
success results with ``isError=False``) and pass once handlers raise
structured errors that FastMCP serialises as error frames.
"""

from unittest.mock import AsyncMock
from uuid import UUID

import pytest

from contexter_server.mcp_server import create_mcp_server
from contexter_server.mcp_tools.errors import HandlerError
from contexter_server.mcp_tools.handlers import (
    handle_agent_resource,
    handle_get_agent_info,
    handle_get_session,
    handle_memory_resource,
    handle_session_resource,
)
from contexter_server.models.agent import Agent
from contexter_server.models.memory import Memory
from contexter_server.models.session import Session


@pytest.fixture
def mock_services():
    return {
        "memory_service": AsyncMock(),
        "session_service": AsyncMock(),
        "agent_service": AsyncMock(),
        "skill_service": AsyncMock(),
        "analytics_service": AsyncMock(),
        "export_service": AsyncMock(),
    }


@pytest.fixture
def live_server(mock_services):
    """Real FastMCP server with AsyncMock services (protocol-level path)."""
    mcp = create_mcp_server(**mock_services)
    assert mcp is not None
    return mcp, mock_services


# ── Handler level: errors must RAISE, never smuggle success frames ──────


class TestHandlerErrorsRaise:
    @pytest.mark.asyncio
    async def test_get_session_not_found_raises_handler_error(self, mock_services):
        """get_session with a nonexistent id must raise, not return {"error": ...}."""
        mock_services["session_service"].get.return_value = None

        with pytest.raises(HandlerError) as exc:
            await handle_get_session(
                id="nonexistent-id",
                session_service=mock_services["session_service"],
            )
        assert "Resource not found: nonexistent-id" in str(exc.value)

    @pytest.mark.asyncio
    async def test_get_session_service_unavailable_raises(self, mock_services):
        """Missing service must raise a structured error, not a success frame."""
        with pytest.raises(HandlerError) as exc:
            await handle_get_session(id="any-id")
        assert "not connected to storage" in str(exc.value)

    @pytest.mark.asyncio
    async def test_get_agent_info_not_found_uses_frozen_message(self, mock_services):
        """Agent not-found must use the same Resource not found: <id> convention."""
        mock_services["agent_service"].get.return_value = None

        with pytest.raises(HandlerError) as exc:
            await handle_get_agent_info(
                id="agent-42",
                agent_service=mock_services["agent_service"],
            )
        assert "Resource not found: agent-42" in str(exc.value)

    @pytest.mark.asyncio
    async def test_resource_not_found_raises_structured_error(self, mock_services):
        """Resource handlers must not return "Session not found" strings as content."""
        mock_services["session_service"].get.return_value = None

        with pytest.raises(HandlerError) as exc:
            await handle_session_resource(
                id="missing-resource",
                session_service=mock_services["session_service"],
            )
        assert "Resource not found: missing-resource" in str(exc.value)

    @pytest.mark.asyncio
    async def test_memory_resource_not_found_raises_structured_error(self, mock_services):
        mock_services["memory_service"].get.return_value = None

        with pytest.raises(HandlerError) as exc:
            await handle_memory_resource(
                id="mem-7",
                memory_service=mock_services["memory_service"],
            )
        assert "Resource not found: mem-7" in str(exc.value)

    @pytest.mark.asyncio
    async def test_agent_resource_not_found_raises_structured_error(self, mock_services):
        mock_services["agent_service"].get.return_value = None

        with pytest.raises(HandlerError) as exc:
            await handle_agent_resource(
                id="agent-9",
                agent_service=mock_services["agent_service"],
            )
        assert "Resource not found: agent-9" in str(exc.value)


# ── Protocol level: isError=True frames, no success smuggling ───────────


class TestLiveErrorFrames:
    @pytest.mark.asyncio
    async def test_get_session_not_found_produces_error_frame(self, live_server):
        """AC-ES-001: get_session with a nonexistent id → isError=True frame."""
        mcp, services = live_server
        services["session_service"].get.return_value = None

        from fastmcp import Client

        async with Client(mcp) as client:
            result = await client.call_tool_mcp(
                "get_session", {"id": "deadbeef-0000-0000-0000-000000000000"}
            )

        assert result.isError is True, (
            "not-found must be an MCP error frame, got a success frame"
        )
        text = result.content[0].text
        assert "Resource not found: deadbeef-0000-0000-0000-000000000000" in text

    @pytest.mark.asyncio
    async def test_error_then_success_sequence_survives(self, live_server):
        """EC-ES-006: error call must not corrupt state for the next call."""
        mcp, services = live_server
        session = Session(
            id=UUID("00000000-0000-0000-0000-000000000001"),
            agent_id=UUID("00000000-0000-0000-0000-000000000001"),
            project="test-project",
            name="Test Session",
        )
        services["session_service"].get.return_value = None
        from fastmcp import Client

        async with Client(mcp) as client:
            err = await client.call_tool_mcp(
                "get_session", {"id": "00000000-0000-0000-0000-000000000099"}
            )
            assert err.isError is True

            services["session_service"].get.return_value = session
            ok = await client.call_tool_mcp(
                "get_session", {"id": "00000000-0000-0000-0000-000000000001"}
            )
            assert ok.isError is False
            assert "Test Session" in ok.content[0].text


# ── Auth must remain byte-identical (REQ-ES-002) ─────────────────────────


class TestAuthShapeUnchanged:
    @pytest.mark.asyncio
    async def test_auth_error_serialisation_unchanged(self, mock_services):
        """MCPAuthError is still raised directly by handlers (auth semantics frozen)."""
        import os
        from unittest import mock

        from contexter_server.mcp_tools.auth import MCPAuthError

        with mock.patch.dict(os.environ, {"CONTEXTER_API_KEY": "test-key-123"}):
            with pytest.raises(MCPAuthError, match="API key required"):
                await handle_get_session(
                    id="00000000-0000-0000-0000-000000000001",
                    session_service=mock_services["session_service"],
                )
