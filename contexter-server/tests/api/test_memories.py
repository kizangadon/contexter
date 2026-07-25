"""Tests for the Memories API router."""

from unittest.mock import AsyncMock

import pytest
from fastapi.testclient import TestClient


class TestListMemories:
    def test_list_memories_empty(self, client: TestClient):
        resp = client.get("/api/v1/memories")
        assert resp.status_code == 200
        assert resp.json() == []

    def test_list_memories_with_data(self, client: TestClient, mock_engine: AsyncMock, sample_memory: dict):
        mock_engine.search_memories.return_value = [sample_memory]
        resp = client.get("/api/v1/memories")
        assert resp.status_code == 200
        data = resp.json()
        assert len(data) == 1
        assert data[0]["role"] == "user"


class TestCreateMemory:
    def test_create_memory_201(self, client: TestClient, mock_engine: AsyncMock, any_uuid: str):
        mock_engine.create_memory.return_value = {
            "id": any_uuid,
            "session_id": any_uuid,
            "agent_id": any_uuid,
            "role": "user",
            "content": "Hello",
            "created_at": "2026-07-25T10:00:00Z",
            "metadata": {},
        }
        resp = client.post("/api/v1/memories", json={
            "session_id": any_uuid,
            "agent_id": any_uuid,
            "role": "user",
            "content": "Hello",
        })
        assert resp.status_code == 201
        assert resp.json()["content"] == "Hello"

    def test_create_memory_422(self, client: TestClient):
        resp = client.post("/api/v1/memories", json={})
        assert resp.status_code == 422


class TestSearchMemories:
    def test_search_memories(self, client: TestClient, mock_engine: AsyncMock, sample_memory: dict):
        mock_engine.search_memories.return_value = [sample_memory]
        mock_engine.count_memories.return_value = 1
        resp = client.get("/api/v1/memories/search?q=hello")
        assert resp.status_code == 200
        data = resp.json()
        assert data["total"] == 1
        assert len(data["results"]) == 1

    def test_search_memories_no_query_422(self, client: TestClient):
        resp = client.get("/api/v1/memories/search")
        assert resp.status_code == 422


class TestGetMemory:
    def test_get_memory_found(self, client: TestClient, mock_engine: AsyncMock, sample_memory: dict):
        mock_engine.get_memory.return_value = sample_memory
        resp = client.get(f"/api/v1/memories/{sample_memory['id']}")
        assert resp.status_code == 200

    def test_get_memory_404(self, client: TestClient):
        resp = client.get("/api/v1/memories/nonexistent")
        assert resp.status_code == 404


class TestUpdateMemory:
    def test_update_memory(self, client: TestClient, mock_engine: AsyncMock, sample_memory: dict):
        updated = dict(sample_memory, content="Updated content")
        mock_engine.update_memory.return_value = updated
        resp = client.put(f"/api/v1/memories/{sample_memory['id']}", json={"content": "Updated content"})
        assert resp.status_code == 200
        assert resp.json()["content"] == "Updated content"

    def test_update_memory_404(self, client: TestClient, mock_engine: AsyncMock):
        mock_engine.update_memory.return_value = None
        resp = client.put("/api/v1/memories/nonexistent", json={"content": "Nope"})
        assert resp.status_code == 404


class TestDeleteMemory:
    def test_delete_memory_204(self, client: TestClient):
        resp = client.delete("/api/v1/memories/some-id")
        assert resp.status_code == 204

    def test_delete_memory_idempotent(self, client: TestClient):
        resp = client.delete("/api/v1/memories/nonexistent")
        assert resp.status_code == 204


class TestCreateMemoryVersion:
    def test_create_version_201(self, client: TestClient, mock_engine: AsyncMock, sample_memory: dict):
        mock_engine.get_memory.return_value = sample_memory
        resp = client.post(f"/api/v1/memories/{sample_memory['id']}/versions")
        assert resp.status_code == 201
        assert resp.json()["status"] == "version_created"

    def test_create_version_404(self, client: TestClient, mock_engine: AsyncMock):
        mock_engine.get_memory.return_value = None
        resp = client.post("/api/v1/memories/nonexistent/versions")
        assert resp.status_code == 404
