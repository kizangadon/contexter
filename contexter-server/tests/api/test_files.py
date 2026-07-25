"""Tests for the Files API router and model validation."""

import pytest
from pydantic import ValidationError

from contexter_server.api.files import WatchFilesRequest


class TestWatchFilesRequestModel:
    """Model validation tests for WatchFilesRequest."""

    def test_valid_watch_request(self):
        """A WatchFilesRequest with required fields passes."""
        req = WatchFilesRequest(path="/tmp")
        assert req.path == "/tmp"
        assert req.recursive is False
        assert req.events == ["create", "modify"]

    def test_valid_watch_request_full(self):
        """A WatchFilesRequest with all fields constructs correctly."""
        req = WatchFilesRequest(path="/home", recursive=True, events=["create", "delete"])
        assert req.path == "/home"
        assert req.recursive is True
        assert req.events == ["create", "delete"]

    def test_watch_request_missing_path(self):
        """WatchFilesRequest without path raises ValidationError."""
        with pytest.raises(ValidationError) as exc:
            WatchFilesRequest()
        assert "path" in str(exc.value)

    def test_watch_request_empty_path(self):
        """WatchFilesRequest with empty path is rejected."""
        with pytest.raises(ValidationError):
            WatchFilesRequest(path="")

    def test_watch_request_empty_events_list(self):
        """WatchFilesRequest with an empty events list is rejected."""
        with pytest.raises(ValidationError):
            WatchFilesRequest(path="/tmp", events=[])


class TestFilesAPI:
    """API endpoint tests for files router."""

    def test_list_files(self, client):
        resp = client.get("/api/v1/files?path=.")
        assert resp.status_code == 200
        assert "files" in resp.json()

    def test_file_diff(self, client):
        resp = client.get("/api/v1/files/abc123/diff?base=abc&compare=def")
        assert resp.status_code == 200
        assert resp.json()["hash"] == "abc123"

    def test_file_diff_missing_params_422(self, client):
        resp = client.get("/api/v1/files/abc123/diff")
        assert resp.status_code == 422

    def test_watch_files(self, client):
        resp = client.post("/api/v1/files/watch", json={"path": "/tmp"})
        assert resp.status_code == 200
        assert resp.json()["status"] == "watching"

    def test_watch_files_missing_path_422(self, client):
        resp = client.post("/api/v1/files/watch", json={})
        assert resp.status_code == 422

    def test_watch_files_empty_path_422(self, client):
        resp = client.post("/api/v1/files/watch", json={"path": ""})
        assert resp.status_code == 422
