"""Tests for the Notifications API router."""

from fastapi.testclient import TestClient


class TestNotifications:
    def test_list_notifications(self, client: TestClient):
        resp = client.get("/api/v1/notifications")
        assert resp.status_code == 200
        data = resp.json()
        assert "notifications" in data
        assert "unread_count" in data

    def test_mark_read_404(self, client: TestClient):
        resp = client.put("/api/v1/notifications/nonexistent/read")
        assert resp.status_code == 404

    def test_mark_all_read(self, client: TestClient):
        resp = client.post("/api/v1/notifications/read-all")
        assert resp.status_code == 200
        assert resp.json()["unread_count"] == 0
