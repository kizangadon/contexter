"""Tests for the Search API router."""

from unittest.mock import AsyncMock

from fastapi.testclient import TestClient


class TestSearch:
    def test_search_with_results(self, client: TestClient, mock_engine: AsyncMock, any_uuid: str):
        mock_engine.search_memories.return_value = [
            {
                "id": any_uuid,
                "session_id": any_uuid,
                "agent_id": any_uuid,
                "role": "user",
                "content": "Hello world",
                "score": 0.95,
            }
        ]
        resp = client.get("/api/v1/search?q=hello")
        assert resp.status_code == 200
        data = resp.json()
        assert data["total"] > 0
        assert len(data["results"]) > 0

    def test_search_empty(self, client: TestClient, mock_engine: AsyncMock):
        mock_engine.search_memories.return_value = []
        resp = client.get("/api/v1/search?q=nonexistent")
        assert resp.status_code == 200
        assert resp.json()["results"] == []
        assert resp.json()["total"] == 0

    def test_search_missing_query_422(self, client: TestClient):
        resp = client.get("/api/v1/search")
        assert resp.status_code == 422

    def test_search_with_project_filter(self, client: TestClient, mock_engine: AsyncMock, any_uuid: str):
        mock_engine.search_memories.return_value = []
        mock_engine.list_sessions.return_value = [
            {"id": any_uuid, "agent_id": any_uuid, "project": "test-project", "name": "Test"}
        ]
        resp = client.get("/api/v1/search?q=test&project=test-project")
        assert resp.status_code == 200
