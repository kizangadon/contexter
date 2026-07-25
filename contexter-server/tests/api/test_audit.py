"""Tests for the Audit API router."""

from unittest.mock import AsyncMock

from fastapi.testclient import TestClient


class TestAudit:
    def test_query_audit_empty(self, client: TestClient, mock_engine: AsyncMock):
        mock_engine.query_audit.return_value = []
        resp = client.get("/api/v1/audit")
        assert resp.status_code == 200
        assert resp.json() == []

    def test_query_audit_with_filters(self, client: TestClient, mock_engine: AsyncMock, any_uuid: str):
        mock_engine.query_audit.return_value = [
            {
                "id": any_uuid,
                "entity_type": "session",
                "entity_id": any_uuid,
                "action": "created",
                "timestamp": "2026-07-25T10:00:00Z",
                "details": {},
            }
        ]
        resp = client.get("/api/v1/audit?entity_type=session&action=created")
        assert resp.status_code == 200
        data = resp.json()
        assert len(data) == 1
        assert data[0]["action"] == "created"

    def test_query_audit_with_limit(self, client: TestClient, mock_engine: AsyncMock):
        mock_engine.query_audit.return_value = []
        resp = client.get("/api/v1/audit?limit=10")
        assert resp.status_code == 200
