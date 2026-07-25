"""Tests for the Analytics API router."""

from unittest.mock import AsyncMock

from fastapi.testclient import TestClient


class TestAnalytics:
    def test_overview(self, client: TestClient, mock_engine: AsyncMock):
        mock_engine.cache_telemetry.return_value = {
            "total_sessions": 10,
            "total_memories": 100,
            "total_agents": 3,
            "total_skills": 5,
            "cache_entries": 50,
            "avg_response_time_ms": 12.5,
            "total_operations": 1000,
            "cache_hit_rate": 0.85,
        }
        mock_engine.storage_size.return_value = {"total_bytes": 1024}
        mock_engine.status.return_value = {
            "status": "ok",
            "uptime_seconds": 3600,
            "memory_usage_mb": 128.0,
            "latency_ms": 5.0,
            "cpu_percent": 25.0,
        }
        resp = client.get("/api/v1/analytics/overview")
        assert resp.status_code == 200
        data = resp.json()
        assert data["total_sessions"] == 10
        assert data["total_memories"] == 100

    def test_health(self, client: TestClient, mock_engine: AsyncMock):
        mock_engine.status.return_value = {"status": "ok", "uptime_seconds": 3600, "memory_usage_mb": 128.0}
        mock_engine.cache_telemetry.return_value = {"cache_entries": 50}
        mock_engine.storage_size.return_value = {"total_bytes": 1024}
        resp = client.get("/api/v1/analytics/health")
        assert resp.status_code == 200
        assert resp.json()["status"] == "ok"

    def test_performance(self, client: TestClient, mock_engine: AsyncMock):
        mock_engine.cache_telemetry.return_value = {
            "avg_response_time_ms": 15.0,
            "total_operations": 500,
            "cache_hit_rate": 0.9,
        }
        resp = client.get("/api/v1/analytics/performance")
        assert resp.status_code == 200

    def test_resources(self, client: TestClient, mock_engine: AsyncMock):
        mock_engine.storage_size.return_value = {"total_bytes": 1048576}
        mock_engine.status.return_value = {"cpu_percent": 30.0, "memory_usage_mb": 256.0}
        mock_engine.cache_telemetry.return_value = {}
        resp = client.get("/api/v1/analytics/resources")
        assert resp.status_code == 200

    def test_costs(self, client: TestClient):
        resp = client.get("/api/v1/analytics/costs")
        assert resp.status_code == 200
        assert resp.json()["total_cost"] == 0.0

    def test_model_cost(self, client: TestClient):
        resp = client.get("/api/v1/analytics/costs/models/gpt-4o")
        assert resp.status_code == 200
        assert resp.json()["model"] == "gpt-4o"

    def test_service_status(self, client: TestClient, mock_engine: AsyncMock):
        mock_engine.status.return_value = {"status": "ok", "latency_ms": 5.0}
        resp = client.get("/api/v1/analytics/services")
        assert resp.status_code == 200
        assert resp.json()["name"] == "contexter-server"
