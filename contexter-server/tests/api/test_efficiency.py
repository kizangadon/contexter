"""Tests for the Efficiency API router."""

from fastapi.testclient import TestClient


class TestEfficiency:
    def test_overview(self, client: TestClient):
        resp = client.get("/api/v1/efficiency/overview")
        assert resp.status_code == 200

    def test_memory(self, client: TestClient):
        resp = client.get("/api/v1/efficiency/memory")
        assert resp.status_code == 200

    def test_sessions(self, client: TestClient):
        resp = client.get("/api/v1/efficiency/sessions")
        assert resp.status_code == 200

    def test_agents(self, client: TestClient):
        resp = client.get("/api/v1/efficiency/agents")
        assert resp.status_code == 200

    def test_skills(self, client: TestClient):
        resp = client.get("/api/v1/efficiency/skills")
        assert resp.status_code == 200

    def test_tokens(self, client: TestClient):
        resp = client.get("/api/v1/efficiency/tokens")
        assert resp.status_code == 200

    def test_correlation(self, client: TestClient):
        resp = client.get("/api/v1/efficiency/correlation")
        assert resp.status_code == 200
