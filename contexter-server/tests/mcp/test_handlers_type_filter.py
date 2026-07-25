"""Tests for MCP handler fixes — type_filter rename, UUID error handling."""

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


class TestTypeFilterRename:
    """Verify type parameter renamed to type_filter to avoid shadowing built-in type()."""

    @pytest.mark.asyncio
    async def test_search_memories_accepts_type_filter(self, mock_services):
        """handle_search_memories should use type_filter parameter."""
        mock_services["memory_service"].search.return_value = SearchResponse()

        result = await handle_search_memories(
            query="test",
            type_filter="user",
            memory_service=mock_services["memory_service"],
        )

        assert "error" not in result

    @pytest.mark.asyncio
    async def test_list_skills_accepts_type_filter(self, mock_services):
        """handle_list_skills should use type_filter parameter."""
        mock_services["skill_service"].list.return_value = []

        result = await handle_list_skills(
            type_filter="memory",
            skill_service=mock_services["skill_service"],
        )

        assert "error" not in result
        call_kwargs = mock_services["skill_service"].list.call_args[1]
        assert call_kwargs["filter"] is not None


class TestUUIDErrorHandling:
    """Verify ValueError from UUID(...) parsing is caught and returned as descriptive error."""

    @pytest.mark.asyncio
    async def test_store_memory_handles_invalid_uuid(self, mock_services):
        """Invalid session_id UUID should return error dict, not propagate 500."""
        mock_services["session_service"].get.return_value = Session(
            id=UUID("00000000-0000-0000-0000-000000000001"),
            agent_id=UUID("00000000-0000-0000-0000-000000000001"),
            project="test",
        )

        result = await handle_store_memory(
            session_id="not-a-uuid",
            role="user",
            content="test",
            memory_service=mock_services["memory_service"],
            session_service=mock_services["session_service"],
        )

        assert "error" in result
        assert "invalid" in result["error"].lower() or "uuid" in result["error"].lower()
