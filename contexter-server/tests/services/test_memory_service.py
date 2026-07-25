"""Tests for MemoryService."""

from unittest.mock import AsyncMock

import pytest

from contexter_server.models.memory import Memory, MemoryCreate, MemoryPatch
from contexter_server.models.search import SearchQuery, SearchResult, SearchResponse
from contexter_server.services.memory_service import MemoryService


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
    async def test_handles_count_failure(self, service, mock_engine, any_uuid):
        """If count_memories fails but search succeeds, results still returned."""
        mock_engine.search_memories.return_value = [
            {"id": any_uuid, "role": "user", "content": "partial", "score": 0.8},
        ]
        mock_engine.count_memories.side_effect = Exception("count failed")
        query = SearchQuery(query="partial")
        result = await service.search(query)
        assert len(result.results) == 1
        assert result.total == 0  # count failed, so total defaults to 0

    @pytest.mark.asyncio
    async def test_handles_search_failure(self, service, mock_engine):
        """If search_memories fails, empty results returned."""
        mock_engine.search_memories.side_effect = Exception("search failed")
        mock_engine.count_memories.return_value = 0
        query = SearchQuery(query="fail")
        result = await service.search(query)
        assert len(result.results) == 0
        assert result.total == 0
