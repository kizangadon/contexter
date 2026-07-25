"""Tests for the Export API router."""

from fastapi.testclient import TestClient


class TestExport:
    def test_submit_export_201(self, client: TestClient):
        resp = client.post("/api/v1/export/submit", json={"format": "json", "entities": ["sessions"]})
        assert resp.status_code == 201
        data = resp.json()
        # ExportService completes synchronously (in-memory)
        assert data["status"] in ("in_progress", "completed")

    def test_submit_export_defaults(self, client: TestClient):
        resp = client.post("/api/v1/export/submit", json={"format": "json"})
        assert resp.status_code == 201

    def test_submit_export_empty_body(self, client: TestClient):
        # ExportRequest has all defaults — {} is valid
        resp = client.post("/api/v1/export/submit", json={})
        assert resp.status_code == 201

    def test_get_export_status_404(self, client: TestClient):
        resp = client.get("/api/v1/export/status/nonexistent")
        assert resp.status_code == 404

    def test_download_export_404(self, client: TestClient):
        resp = client.get("/api/v1/export/download/nonexistent")
        assert resp.status_code == 404

    def test_export_history(self, client: TestClient):
        resp = client.get("/api/v1/export/history")
        assert resp.status_code == 200
