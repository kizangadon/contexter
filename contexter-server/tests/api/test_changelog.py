"""Tests for the Changelog API router."""

from fastapi.testclient import TestClient


class TestChangelog:
    def test_list_changelog(self, client: TestClient):
        resp = client.get("/api/v1/changelog")
        assert resp.status_code == 200
        assert resp.json() == []
