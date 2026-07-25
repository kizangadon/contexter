"""Tests for SearchService."""

from unittest.mock import AsyncMock

import pytest

from contexter_server.models.search import SearchQuery, SearchResponse
from contexter_server.services.search_service import SearchService


@pytest.fixture
def mock_engine():
    engine = AsyncMock()
    return engine


@pytest.fixture
def service(mock_engine):
    return SearchService(mock_engine)


class TestSearchServiceSearch:
    """Tests for SearchService.search."""

    @pytest.mark.asyncio
    async def test_search_returns_memory_results(self, service, mock_engine, any_uuid):
        mid1 = any_uuid.replace("000001", "000002")
        mid2 = any_uuid.replace("000001", "000003")
        mock_engine.search_memories.return_value = [
            {"id": mid1, "content": "test content here", "role": "user", "score": 0.95},
            {"id": mid2, "content": "another result", "role": "assistant", "score": 0.80},
        ]
        query = SearchQuery(query="test", page=1, limit=20)
        result = await service.search(query)
        assert isinstance(result, SearchResponse)
        assert len(result.results) == 2
        assert result.total == 2
        assert result.results[0].type == "memory"
        assert result.results[0].score == 0.95

    @pytest.mark.asyncio
    async def test_search_with_project_also_searches_sessions(self, service, mock_engine, any_uuid):
        mid = any_uuid.replace("000001", "000002")
        sid = any_uuid.replace("000001", "000003")
        mock_engine.search_memories.return_value = [
            {"id": mid, "content": "project content", "score": 0.9},
        ]
        mock_engine.list_sessions.return_value = [
            {"id": sid, "project": "my-project", "name": "Test Session"},
        ]
        query = SearchQuery(query="project", project="my-project", page=1, limit=20)
        result = await service.search(query)
        assert len(result.results) == 2
        assert result.results[0].type == "memory"
        assert result.results[1].type == "session"
        mock_engine.list_sessions.assert_awaited_once()

    @pytest.mark.asyncio
    async def test_search_returns_empty_when_no_match(self, service, mock_engine):
        mock_engine.search_memories.return_value = []
        query = SearchQuery(query="nonexistent")
        result = await service.search(query)
        assert len(result.results) == 0
        assert result.total == 0

    @pytest.mark.asyncio
    async def test_search_respects_pagination(self, service, mock_engine, any_uuid):
        from uuid import uuid4 as _make_uuid
        mock_engine.search_memories.return_value = [
            {"id": str(_make_uuid()), "content": f"item {i}", "score": 1.0 - i * 0.1}
            for i in range(10)
        ]
        query = SearchQuery(query="test", page=2, limit=3)
        result = await service.search(query)
        # After sorting descending by score, page 2 of 3 per page = items at index 3..5
        assert len(result.results) == 3
        assert result.page == 2
        assert result.limit == 3

    @pytest.mark.asyncio
    async def test_gathers_search_and_list_sessions(self, service, mock_engine, any_uuid):
        """Verify search_memories and list_sessions are gathered when project is given."""
        mid = any_uuid.replace("000001", "000002")
        sid = any_uuid.replace("000001", "000003")
        mock_engine.search_memories.return_value = [
            {"id": mid, "content": "gathered", "score": 0.9},
        ]
        mock_engine.list_sessions.return_value = [
            {"id": sid, "project": "my-project", "name": "Test"},
        ]
        query = SearchQuery(query="gathered", project="my-project", page=1, limit=20)
        result = await service.search(query)
        assert len(result.results) == 2

    @pytest.mark.asyncio
    async def test_handles_session_list_failure(self, service, mock_engine, any_uuid):
        """If list_sessions fails but search succeeds, memory results still returned."""
        mid = any_uuid.replace("000001", "000002")
        mock_engine.search_memories.return_value = [
            {"id": mid, "content": "memory-only", "score": 0.9},
        ]
        mock_engine.list_sessions.side_effect = Exception("sessions failed")
        query = SearchQuery(query="memory-only", project="my-project", page=1, limit=20)
        result = await service.search(query)
        assert len(result.results) == 1
        assert result.results[0].type == "memory"
