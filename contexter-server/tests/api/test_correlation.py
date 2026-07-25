"""Tests for the Correlation API router."""

from fastapi.testclient import TestClient


class TestCorrelation:
    def test_overview(self, client: TestClient):
        resp = client.get("/api/v1/correlation/overview?timeframe=24h")
        assert resp.status_code == 200
        assert resp.json()["timeframe_hours"] == 24

    def test_overview_default(self, client: TestClient):
        resp = client.get("/api/v1/correlation/overview")
        assert resp.status_code == 200

    def test_timeline(self, client: TestClient):
        resp = client.get("/api/v1/correlation/timeline")
        assert resp.status_code == 200

    def test_timeline_with_project(self, client: TestClient):
        resp = client.get("/api/v1/correlation/timeline?project=test")
        assert resp.status_code == 200

    def test_compare(self, client: TestClient):
        resp = client.get("/api/v1/correlation/compare?a=entity-a&b=entity-b")
        assert resp.status_code == 200
        data = resp.json()
        assert data["entity_a_id"] == "entity-a"
        assert data["entity_b_id"] == "entity-b"

    def test_compare_missing_params_422(self, client: TestClient):
        resp = client.get("/api/v1/correlation/compare")
        assert resp.status_code == 422
