"""Tests for the Analytics API router.

Engine mocks mirror the real Rust engine shapes: snake_case
``cache_telemetry()`` (``entries_by_type``/``total_ops``), camelCase
``storage_size()`` (``total``/``perCf``/``walSize``), and ``status()``
with nested ``cacheTelemetry``. Store-backed counts come from
``count_sessions``/``count_memories``/``count_agents``/``count_skills``.
"""

from unittest.mock import AsyncMock

import pytest
from fastapi.testclient import TestClient


class TestAnalytics:
    def test_overview(self, client: TestClient, mock_engine: AsyncMock):
        mock_engine.cache_telemetry.return_value = {
            "gets": 0,
            "hits": 0,
            "misses": 0,
            "stores": 3,
            "invalidations": 2,
            "total_ops": 1000,
            "entries_by_type": {"agent": 3, "session": 10, "skill": 5},
        }
        mock_engine.storage_size.return_value = {"perCf": {}, "total": 1024, "walSize": 0}
        mock_engine.status.return_value = {
            "status": "ok",
            "version": "0.1.0",
            "cacheTelemetry": {
                "entriesByType": {},
                "hitRatio": 0.0,
                "hits": 0,
                "misses": 0,
                "totalOps": 0,
            },
        }
        mock_engine.count_sessions.return_value = 10
        mock_engine.count_memories.return_value = 100
        mock_engine.count_agents.return_value = 3
        mock_engine.count_skills.return_value = 5

        resp = client.get("/api/v1/analytics/overview")
        assert resp.status_code == 200
        data = resp.json()
        assert data["total_sessions"] == 10
        assert data["total_memories"] == 100
        assert data["total_agents"] == 3
        assert data["total_skills"] == 5
        assert data["storage_size_bytes"] == 1024

    def test_health(self, client: TestClient, mock_engine: AsyncMock):
        mock_engine.status.return_value = {
            "status": "ok",
            "version": "0.1.0",
            "cacheTelemetry": {
                "entriesByType": {},
                "hitRatio": 0.0,
                "hits": 0,
                "misses": 0,
                "totalOps": 0,
            },
        }
        mock_engine.cache_telemetry.return_value = {
            "gets": 0,
            "hits": 0,
            "misses": 0,
            "stores": 0,
            "invalidations": 0,
            "total_ops": 0,
            "entries_by_type": {"agent": 1, "session": 1, "skill": 1},
        }
        mock_engine.storage_size.return_value = {"perCf": {}, "total": 1024, "walSize": 0}
        resp = client.get("/api/v1/analytics/health")
        assert resp.status_code == 200
        data = resp.json()
        assert data["status"] == "ok"
        assert data["storage_size_bytes"] == 1024
        assert data["cache_entries"] == 3

    def test_performance(self, client: TestClient, mock_engine: AsyncMock):
        mock_engine.cache_telemetry.return_value = {
            "gets": 100,
            "hits": 90,
            "misses": 10,
            "stores": 0,
            "invalidations": 0,
            "total_ops": 500,
            "entries_by_type": {},
        }
        resp = client.get("/api/v1/analytics/performance")
        assert resp.status_code == 200
        data = resp.json()
        assert data["total_operations"] == 500
        assert data["cache_hit_rate"] == pytest.approx(0.9)

    def test_resources(self, client: TestClient, mock_engine: AsyncMock):
        mock_engine.storage_size.return_value = {"perCf": {}, "total": 1048576, "walSize": 0}
        mock_engine.status.return_value = {
            "status": "ok",
            "version": "0.1.0",
            "cacheTelemetry": {
                "entriesByType": {},
                "hitRatio": 0.0,
                "hits": 0,
                "misses": 0,
                "totalOps": 0,
            },
        }
        mock_engine.cache_telemetry.return_value = {
            "gets": 0,
            "hits": 0,
            "misses": 0,
            "stores": 0,
            "invalidations": 0,
            "total_ops": 0,
            "entries_by_type": {},
        }
        resp = client.get("/api/v1/analytics/resources")
        assert resp.status_code == 200
        assert resp.json()["storage_mb"] == 1.0

    def test_costs(self, client: TestClient):
        resp = client.get("/api/v1/analytics/costs")
        assert resp.status_code == 200
        assert resp.json()["total_cost"] == 0.0

    def test_model_cost(self, client: TestClient):
        resp = client.get("/api/v1/analytics/costs/models/gpt-4o")
        assert resp.status_code == 200
        assert resp.json()["model"] == "gpt-4o"

    def test_service_status(self, client: TestClient, mock_engine: AsyncMock):
        mock_engine.status.return_value = {
            "status": "ok",
            "version": "0.1.0",
            "cacheTelemetry": {
                "entriesByType": {},
                "hitRatio": 0.0,
                "hits": 0,
                "misses": 0,
                "totalOps": 0,
            },
        }
        resp = client.get("/api/v1/analytics/services")
        assert resp.status_code == 200
        assert resp.json()["name"] == "contexter-server"
        assert resp.json()["status"] == "ok"
