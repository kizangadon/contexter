"""Tests for search Pydantic models."""

import uuid

import pytest
from pydantic import ValidationError

from contexter_server.models.search import SearchQuery, SearchResult, SearchResponse


class TestSearchModels:
    """Search model validation tests."""

    def test_search_query_defaults(self):
        """SearchQuery should have default pagination."""
        q = SearchQuery(query="hello")
        assert q.query == "hello"
        assert q.type is None
        assert q.project is None
        assert q.page == 1
        assert q.limit == 20

    def test_search_query_custom(self):
        """SearchQuery with custom pagination."""
        q = SearchQuery(query="test", type="memory", project="my-project", page=2, limit=50)
        assert q.type == "memory"
        assert q.project == "my-project"
        assert q.page == 2
        assert q.limit == 50

    def test_search_query_page_ge_1(self):
        """SearchQuery page must be >= 1."""
        with pytest.raises(ValidationError):
            SearchQuery(query="test", page=0)

    def test_search_query_limit_range(self):
        """SearchQuery limit must be 1-100."""
        with pytest.raises(ValidationError):
            SearchQuery(query="test", limit=0)
        with pytest.raises(ValidationError):
            SearchQuery(query="test", limit=101)

    def test_search_result_defaults(self):
        """SearchResult defaults."""
        rid = uuid.uuid4()
        r = SearchResult(id=rid, type="memory")
        assert r.id == rid
        assert r.type == "memory"
        assert r.score == 0.0
        assert r.data == {}
        assert r.snippet is None

    def test_search_result_with_data(self):
        """SearchResult with data and snippet."""
        rid = uuid.uuid4()
        r = SearchResult(
            id=rid,
            type="session",
            score=0.95,
            data={"title": "Test"},
            snippet="Some snippet...",
        )
        assert r.score == 0.95
        assert r.data == {"title": "Test"}
        assert r.snippet == "Some snippet..."

    def test_search_result_score_range(self):
        """Score must be between 0 and 1."""
        rid = uuid.uuid4()
        with pytest.raises(ValidationError):
            SearchResult(id=rid, type="m", score=-0.1)
        with pytest.raises(ValidationError):
            SearchResult(id=rid, type="m", score=1.1)

    def test_search_response_defaults(self):
        """SearchResponse defaults."""
        r = SearchResponse()
        assert r.results == []
        assert r.total == 0
        assert r.page == 1
        assert r.limit == 20
