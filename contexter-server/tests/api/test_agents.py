"""Tests for the Agents API router."""

from unittest.mock import AsyncMock

from fastapi.testclient import TestClient


class TestListAgents:
    def test_list_agents_empty(self, client: TestClient):
        resp = client.get("/api/v1/agents")
        assert resp.status_code == 200
        assert resp.json() == []

    def test_list_agents_with_data(self, client: TestClient, mock_engine: AsyncMock, sample_agent: dict):
        mock_engine.list_agents.return_value = [sample_agent]
        resp = client.get("/api/v1/agents")
        assert resp.status_code == 200
        assert len(resp.json()) == 1


class TestCreateAgent:
    def test_create_agent_201(self, client: TestClient, mock_engine: AsyncMock, any_uuid: str):
        mock_engine.create_agent.return_value = {
            "id": any_uuid,
            "name": "new-agent",
            "provider": "anthropic",
            "model": "claude-3",
            "temperature": 0.7,
            "tools": [],
            "metadata": {},
            "created_at": "2026-07-25T10:00:00Z",
            "updated_at": "2026-07-25T10:00:00Z",
        }
        resp = client.post("/api/v1/agents", json={
            "name": "new-agent",
            "provider": "anthropic",
            "model": "claude-3",
        })
        assert resp.status_code == 201
        assert resp.json()["name"] == "new-agent"

    def test_create_agent_422(self, client: TestClient):
        resp = client.post("/api/v1/agents", json={})
        assert resp.status_code == 422


class TestGetAgent:
    def test_get_agent_found(self, client: TestClient, mock_engine: AsyncMock, sample_agent: dict):
        mock_engine.get_agent.return_value = sample_agent
        resp = client.get(f"/api/v1/agents/{sample_agent['id']}")
        assert resp.status_code == 200

    def test_get_agent_404(self, client: TestClient):
        resp = client.get("/api/v1/agents/nonexistent")
        assert resp.status_code == 404


class TestUpdateAgent:
    def test_update_agent(self, client: TestClient, mock_engine: AsyncMock, sample_agent: dict):
        updated = dict(sample_agent, name="updated-agent")
        mock_engine.update_agent.return_value = updated
        resp = client.put(f"/api/v1/agents/{sample_agent['id']}", json={"name": "updated-agent"})
        assert resp.status_code == 200
        assert resp.json()["name"] == "updated-agent"

    def test_update_agent_404(self, client: TestClient, mock_engine: AsyncMock):
        mock_engine.update_agent.return_value = {}
        resp = client.put("/api/v1/agents/nonexistent", json={"name": "Nope"})
        assert resp.status_code == 404


class TestDeleteAgent:
    def test_delete_agent_204(self, client: TestClient):
        resp = client.delete("/api/v1/agents/some-id")
        assert resp.status_code == 204

    def test_delete_agent_idempotent(self, client: TestClient):
        resp = client.delete("/api/v1/agents/nonexistent")
        assert resp.status_code == 204
