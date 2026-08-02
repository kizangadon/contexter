"""Tests for MemoryService."""

import logging
import re
from unittest.mock import AsyncMock

import pytest

from contexter_server.models.memory import Memory, MemoryCreate, MemoryPatch
from contexter_server.models.search import SearchQuery, SearchResult, SearchResponse
from contexter_server.services.memory_service import MemoryService

_MEMORY_LOGGER = "contexter_server.services.memory_service"

# structlog's ConsoleRenderer interleaves ANSI color codes into the rendered
# message; strip them before asserting on log content.
_ANSI_ESCAPE = re.compile(r"\x1b\[[0-9;]*m")


@pytest.fixture
def mock_engine():
    engine = AsyncMock()
    return engine


@pytest.fixture
def service(mock_engine):
    return MemoryService(mock_engine)


class TestMemoryServiceCreate:
    """Tests for MemoryService.create."""

    @pytest.mark.asyncio
    async def test_creates_memory(self, service, mock_engine, any_uuid):
        mid = any_uuid.replace("000001", "000002")
        mock_engine.create_memory.return_value = {
            "id": mid,
            "session_id": any_uuid,
            "agent_id": any_uuid,
            "role": "user",
            "content": "Hello",
        }
        data = MemoryCreate(session_id=any_uuid, agent_id=any_uuid, role="user", content="Hello")
        result = await service.create(data)
        assert str(result.id) == mid
        assert result.content == "Hello"
        mock_engine.create_memory.assert_awaited_once()


class TestMemoryServiceGet:
    """Tests for MemoryService.get."""

    @pytest.mark.asyncio
    async def test_gets_memory(self, service, mock_engine, any_uuid):
        mock_engine.get_memory.return_value = {
            "id": any_uuid,
            "session_id": any_uuid,
            "agent_id": any_uuid,
            "role": "user",
            "content": "Hello",
        }
        result = await service.get(any_uuid)
        assert result is not None
        assert result.content == "Hello"

    @pytest.mark.asyncio
    async def test_get_returns_none_when_not_found(self, service, mock_engine):
        mock_engine.get_memory.return_value = None
        result = await service.get("nonexistent")
        assert result is None


class TestMemoryServiceList:
    """Tests for MemoryService.list."""

    @pytest.mark.asyncio
    async def test_lists_memories(self, service, mock_engine, any_uuid):
        mid1 = any_uuid.replace("000001", "000002")
        mid2 = any_uuid.replace("000001", "000003")
        mock_engine.search_memories.return_value = [
            {"id": mid1, "session_id": any_uuid, "agent_id": any_uuid, "role": "user", "content": "A"},
            {"id": mid2, "session_id": any_uuid, "agent_id": any_uuid, "role": "assistant", "content": "B"},
        ]
        result = await service.list()
        assert len(result) == 2
        assert result[0].content == "A"

    @pytest.mark.asyncio
    async def test_list_returns_empty(self, service, mock_engine):
        mock_engine.search_memories.return_value = []
        result = await service.list()
        assert result == []


class TestMemoryServiceUpdate:
    """Tests for MemoryService.update."""

    @pytest.mark.asyncio
    async def test_updates_memory(self, service, mock_engine, any_uuid):
        mock_engine.update_memory.return_value = {
            "id": any_uuid,
            "session_id": any_uuid,
            "agent_id": any_uuid,
            "role": "user",
            "content": "Updated",
        }
        patch = MemoryPatch(content="Updated")
        result = await service.update(any_uuid, patch)
        assert result is not None
        assert result.content == "Updated"

    @pytest.mark.asyncio
    async def test_update_returns_none_when_not_found(self, service, mock_engine):
        mock_engine.update_memory.return_value = None
        patch = MemoryPatch(content="Updated")
        result = await service.update("nonexistent", patch)
        assert result is None


class TestMemoryServiceDelete:
    """Tests for MemoryService.delete."""

    @pytest.mark.asyncio
    async def test_deletes_memory(self, service, mock_engine, any_uuid):
        await service.delete(any_uuid)
        mock_engine.delete_memory.assert_awaited_once_with(any_uuid)


class TestMemoryServiceSearch:
    """Tests for MemoryService.search."""

    @pytest.mark.asyncio
    async def test_search_returns_results(self, service, mock_engine, any_uuid):
        mock_engine.search_memories.return_value = [
            {"id": any_uuid, "role": "user", "content": "test content", "score": 0.95},
        ]
        mock_engine.count_memories.return_value = 1
        query = SearchQuery(query="test")
        result = await service.search(query)
        assert isinstance(result, SearchResponse)
        assert len(result.results) == 1
        assert result.total == 1
        assert result.results[0].type == "memory"

    @pytest.mark.asyncio
    async def test_search_returns_empty_when_no_match(self, service, mock_engine):
        mock_engine.search_memories.return_value = []
        mock_engine.count_memories.return_value = 0
        query = SearchQuery(query="nonexistent")
        result = await service.search(query)
        assert len(result.results) == 0
        assert result.total == 0

    @pytest.mark.asyncio
    async def test_gathers_search_and_count(self, service, mock_engine, any_uuid):
        """Verify search_memories and count_memories are gathered concurrently."""
        mock_engine.search_memories.return_value = [
            {"id": any_uuid, "role": "user", "content": "gathered", "score": 0.9},
        ]
        mock_engine.count_memories.return_value = 1
        query = SearchQuery(query="gathered")
        result = await service.search(query)
        assert len(result.results) == 1
        assert result.total == 1

    @pytest.mark.asyncio
    async def test_count_failure_surfaces_negative_total(
        self, service, mock_engine, any_uuid
    ):
        """REQ-STF-001: a count failure must never silently report total=0.

        Results are still returned, but the total surfaces a distinguishing
        signal (-1) so callers can tell the count is unknown.
        """
        mock_engine.search_memories.return_value = [
            {"id": any_uuid, "role": "user", "content": "partial", "score": 0.8},
        ]
        mock_engine.count_memories.side_effect = Exception("count failed")
        query = SearchQuery(query="partial")
        result = await service.search(query)
        assert len(result.results) == 1
        assert result.total == -1  # distinguishing signal, never a silent 0

    @pytest.mark.asyncio
    async def test_count_failure_logs_explicit_error(
        self, service, mock_engine, any_uuid, caplog
    ):
        """REQ-STF-001: a count failure is logged explicitly, never masked."""
        mock_engine.search_memories.return_value = [
            {"id": any_uuid, "role": "user", "content": "partial", "score": 0.8},
        ]
        mock_engine.count_memories.side_effect = Exception("count failed")
        query = SearchQuery(query="partial")

        with caplog.at_level(logging.ERROR, logger=_MEMORY_LOGGER):
            result = await service.search(query)

        assert result.total == -1
        # structlog's stdlib LoggerFactory renders the full line (event +
        # key-value pairs) before the stdlib call, so assert on the message.
        messages = [
            _ANSI_ESCAPE.sub("", r.getMessage())
            for r in caplog.records
            if r.name == _MEMORY_LOGGER
        ]
        assert any("search_count_failed" in m for m in messages)

    @pytest.mark.asyncio
    async def test_search_failure_propagates_error(self, service, mock_engine):
        """EC-STF-001: a failed results call is an error, not silent empty results."""
        mock_engine.search_memories.side_effect = Exception("search failed")
        mock_engine.count_memories.return_value = 0
        query = SearchQuery(query="fail")
        with pytest.raises(Exception, match="search failed"):
            await service.search(query)

    @pytest.mark.asyncio
    async def test_both_calls_fail_propagates_error(self, service, mock_engine):
        """EC-STF-002: results and count both failing follows the error path."""
        mock_engine.search_memories.side_effect = Exception("search failed")
        mock_engine.count_memories.side_effect = Exception("count failed")
        query = SearchQuery(query="fail")
        with pytest.raises(Exception, match="search failed"):
            await service.search(query)

    @pytest.mark.asyncio
    async def test_total_reflects_full_count_when_truncated(
        self, service, mock_engine, any_uuid
    ):
        """EC-STF-004: total is the real count even when the page is truncated."""
        mock_engine.search_memories.return_value = [
            {"id": any_uuid, "role": "user", "content": "one", "score": 0.9},
        ]
        mock_engine.count_memories.return_value = 42
        query = SearchQuery(query="truncated", limit=1)
        result = await service.search(query)
        assert len(result.results) == 1
        assert result.total == 42
        assert result.total >= len(result.results)
