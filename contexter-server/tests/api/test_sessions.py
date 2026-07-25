"""Tests for the Sessions API router."""

from unittest.mock import AsyncMock

import pytest
from fastapi.testclient import TestClient


class TestListSessions:
    def test_list_sessions_empty(self, client: TestClient):
        resp = client.get("/api/v1/sessions")
        assert resp.status_code == 200
        assert resp.json() == []

    def test_list_sessions_with_data(self, client: TestClient, mock_engine: AsyncMock, sample_session: dict):
        mock_engine.list_sessions.return_value = [sample_session]
        resp = client.get("/api/v1/sessions")
        assert resp.status_code == 200
        data = resp.json()
        assert len(data) == 1
        assert data[0]["project"] == "test-project"

    def test_list_sessions_filtered(self, client: TestClient, mock_engine: AsyncMock, sample_session: dict):
        mock_engine.list_sessions.return_value = [sample_session]
        resp = client.get("/api/v1/sessions?project=test-project")
        assert resp.status_code == 200


class TestCreateSession:
    def test_create_session_201(self, client: TestClient, mock_engine: AsyncMock, any_uuid: str):
        mock_engine.create_session.return_value = {
            "id": any_uuid,
            "agent_id": any_uuid,
            "project": "test-project",
            "name": "New Session",
            "status": "active",
            "started_at": "2026-07-25T10:00:00Z",
            "updated_at": "2026-07-25T10:00:00Z",
            "completed_at": None,
            "metadata": {},
        }
        resp = client.post("/api/v1/sessions", json={
            "agent_id": any_uuid,
            "project": "test-project",
            "name": "New Session",
        })
        assert resp.status_code == 201
        data = resp.json()
        assert data["project"] == "test-project"

    def test_create_session_422_missing_fields(self, client: TestClient):
        resp = client.post("/api/v1/sessions", json={})
        assert resp.status_code == 422


class TestGetSession:
    def test_get_session_found(self, client: TestClient, mock_engine: AsyncMock, sample_session: dict):
        mock_engine.get_session.return_value = sample_session
        resp = client.get(f"/api/v1/sessions/{sample_session['id']}")
        assert resp.status_code == 200
        assert resp.json()["id"] == sample_session["id"]

    def test_get_session_404(self, client: TestClient):
        resp = client.get("/api/v1/sessions/nonexistent")
        assert resp.status_code == 404


class TestUpdateSession:
    def test_update_session(self, client: TestClient, mock_engine: AsyncMock, sample_session: dict):
        updated = dict(sample_session, name="Updated", status="paused")
        mock_engine.update_session.return_value = updated
        resp = client.put(f"/api/v1/sessions/{sample_session['id']}", json={"name": "Updated", "status": "paused"})
        assert resp.status_code == 200
        assert resp.json()["name"] == "Updated"

    def test_update_session_404(self, client: TestClient):
        mock_engine = client.app.state.storage_engine
        mock_engine.update_session.return_value = {}
        resp = client.put("/api/v1/sessions/nonexistent", json={"name": "Nope"})
        assert resp.status_code == 404


class TestDeleteSession:
    def test_delete_session_204(self, client: TestClient):
        resp = client.delete("/api/v1/sessions/some-id")
        assert resp.status_code == 204

    def test_delete_session_idempotent(self, client: TestClient):
        resp = client.delete("/api/v1/sessions/nonexistent")
        assert resp.status_code == 204


class TestResumeSession:
    def test_resume_session(self, client: TestClient, mock_engine: AsyncMock, sample_session: dict):
        resumed = dict(sample_session, status="active")
        mock_engine.update_session.return_value = resumed
        resp = client.post(f"/api/v1/sessions/{sample_session['id']}/resume")
        assert resp.status_code == 200
        assert resp.json()["status"] == "active"

    def test_resume_session_404(self, client: TestClient, mock_engine: AsyncMock):
        mock_engine.update_session.return_value = {}
        resp = client.post("/api/v1/sessions/nonexistent/resume")
        assert resp.status_code == 404
