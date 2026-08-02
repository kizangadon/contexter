"""Tests for MCP handler fixes — type parameter, UUID error handling."""

from unittest.mock import AsyncMock
from uuid import UUID

import pytest

from contexter_server.mcp_tools.handlers import (
    handle_search_memories,
    handle_list_skills,
    handle_store_memory,
)
from contexter_server.models.search import SearchResponse
from contexter_server.models.session import Session


@pytest.fixture
def mock_services():
    return {
        "memory_service": AsyncMock(),
        "session_service": AsyncMock(),
        "skill_service": AsyncMock(),
    }


class TestTypeParameter:
    """Verify list_skills/search_memories accept the ``type`` parameter (SPEC AC-003)."""

    @pytest.mark.asyncio
    async def test_search_memories_accepts_type(self, mock_services):
        """handle_search_memories should use the type parameter."""
        mock_services["memory_service"].search.return_value = SearchResponse()

        result = await handle_search_memories(
            query="test",
            type="user",
            memory_service=mock_services["memory_service"],
        )

        assert "error" not in result

    @pytest.mark.asyncio
    async def test_list_skills_accepts_type(self, mock_services):
        """handle_list_skills should use the type parameter."""
        mock_services["skill_service"].list.return_value = []

        result = await handle_list_skills(
            type="memory",
            skill_service=mock_services["skill_service"],
        )

        assert "error" not in result
        call_kwargs = mock_services["skill_service"].list.call_args[1]
        assert call_kwargs["filter"] is not None


class TestUUIDErrorHandling:
    """Verify ValueError from UUID(...) parsing becomes a structured error."""

    @pytest.mark.asyncio
    async def test_store_memory_handles_invalid_uuid(self, mock_services):
        """Invalid session_id UUID raises HandlerError, never a 500 or error dict."""
        from contexter_server.mcp_tools.errors import HandlerError

        mock_services["session_service"].get.return_value = Session(
            id=UUID("00000000-0000-0000-0000-000000000001"),
            agent_id=UUID("00000000-0000-0000-0000-000000000001"),
            project="test",
        )

        with pytest.raises(HandlerError) as exc:
            await handle_store_memory(
                session_id="not-a-uuid",
                role="user",
                content="test",
                memory_service=mock_services["memory_service"],
                session_service=mock_services["session_service"],
            )

        assert "invalid" in str(exc.value).lower() or "uuid" in str(exc.value).lower()
