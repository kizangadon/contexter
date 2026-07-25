"""Tests for security middleware hardening (Bug 8).

Covers: API key auth, security headers, docs gating, body size limiting,
TrustedHostMiddleware, path traversal protection, and debug mode.
"""

import os
from unittest import mock

import pytest
from fastapi.testclient import TestClient

# ---------------------------------------------------------------------------
# 8a: API Key Authentication Middleware
# ---------------------------------------------------------------------------


class TestApiKeyAuth:
    """8a: API Key Authentication Middleware."""

    def test_health_without_auth_when_key_configured(self, client: TestClient) -> None:
        """Health endpoint is accessible without auth even when API key is set."""
        with mock.patch.dict(os.environ, {"CONtexTER_API_KEY": "test-key-123"}):
            resp = client.get("/health")
        assert resp.status_code == 200

    def test_api_v1_rejects_missing_key(self, client: TestClient) -> None:
        """API v1 endpoint rejects request without auth when key configured."""
        with mock.patch.dict(os.environ, {"CONtexTER_API_KEY": "test-key-123"}):
            resp = client.get("/api/v1/sessions")
        assert resp.status_code == 401

    def test_api_v1_rejects_wrong_key(self, client: TestClient) -> None:
        """API v1 endpoint rejects request with wrong bearer token."""
        with mock.patch.dict(os.environ, {"CONtexTER_API_KEY": "test-key-123"}):
            resp = client.get(
                "/api/v1/sessions",
                headers={"Authorization": "Bearer wrong-key"},
            )
        assert resp.status_code == 401

    def test_api_v1_accepts_valid_key(self, client: TestClient) -> None:
        """API v1 endpoint accepts request with correct bearer token."""
        with mock.patch.dict(os.environ, {"CONtexTER_API_KEY": "test-key-123"}):
            resp = client.get(
                "/api/v1/sessions",
                headers={"Authorization": "Bearer test-key-123"},
            )
        assert resp.status_code == 200

    def test_no_key_configured_allows_all(self, client: TestClient) -> None:
        """When no API key env var is set, all requests pass without auth."""
        old = os.environ.pop("CONtexTER_API_KEY", None)
        try:
            resp = client.get("/api/v1/sessions")
            assert resp.status_code == 200
        finally:
            if old is not None:
                os.environ["CONtexTER_API_KEY"] = old

    def test_multiple_routers_receive_auth(self, client: TestClient) -> None:
        """Several /api/v1/ routers have the auth dependency applied."""
        endpoints: list[tuple[str, str]] = [
            ("GET", "/api/v1/sessions"),
            ("GET", "/api/v1/memories"),
            ("GET", "/api/v1/agents"),
            ("GET", "/api/v1/skills"),
            ("GET", "/api/v1/analytics/overview"),
            ("GET", "/api/v1/efficiency/overview"),
            ("GET", "/api/v1/settings/project"),
            ("GET", "/api/v1/notifications"),
            ("GET", "/api/v1/audit"),
            ("GET", "/api/v1/files"),
            ("GET", "/api/v1/correlation/overview"),
            ("GET", "/api/v1/changelog"),
        ]
        with mock.patch.dict(os.environ, {"CONtexTER_API_KEY": "test-key-123"}):
            for method, path in endpoints:
                resp = client.request(method, path)
                assert resp.status_code == 401, (
                    f"{method} {path} returned {resp.status_code}, expected 401"
                )


# ---------------------------------------------------------------------------
# 8b: Security Headers Middleware
# ---------------------------------------------------------------------------


class TestSecurityHeaders:
    """8b: Security Headers Middleware."""

    EXPECTED_HEADERS: dict[str, str] = {
        "x-content-type-options": "nosniff",
        "x-frame-options": "DENY",
        "content-security-policy": "default-src 'self'",
        "referrer-policy": "no-referrer",
    }

    def test_headers_on_health_endpoint(self, client: TestClient) -> None:
        """All required security headers are present on GET /health."""
        resp = client.get("/health")
        for header, expected in self.EXPECTED_HEADERS.items():
            assert resp.headers.get(header) == expected, (
                f"Missing or wrong {header}: expected {expected!r}, got {resp.headers.get(header)!r}"
            )

    def test_headers_on_api_endpoint(self, client: TestClient) -> None:
        """All required security headers are present on API responses."""
        resp = client.get("/api/v1/sessions")
        for header, expected in self.EXPECTED_HEADERS.items():
            assert resp.headers.get(header) == expected, (
                f"Missing or wrong {header}: expected {expected!r}, got {resp.headers.get(header)!r}"
            )

    def test_headers_on_404_response(self, client: TestClient) -> None:
        """Security headers are present even on error responses."""
        resp = client.get("/nonexistent")
        for header, expected in self.EXPECTED_HEADERS.items():
            assert resp.headers.get(header) == expected, (
                f"Missing {header} on 404 response"
            )


# ---------------------------------------------------------------------------
# 8c: Gate OpenAPI Docs
# ---------------------------------------------------------------------------


class TestOpenApiDocs:
    """8c: Gate OpenAPI Docs (disabled by default)."""

    def test_docs_disabled_by_default(self, client: TestClient) -> None:
        """When CONtexTER_ENABLE_DOCS is not set, docs endpoints return 404."""
        env_val = os.environ.get("CONtexTER_ENABLE_DOCS")
        if env_val and env_val.strip().lower() == "true":
            pytest.skip("CONtexTER_ENABLE_DOCS is active in this environment")
        assert client.get("/docs").status_code == 404
        assert client.get("/redoc").status_code == 404
        assert client.get("/openapi.json").status_code == 404

    def test_docs_enabled_with_env_var(self, monkeypatch: pytest.MonkeyPatch) -> None:
        """When CONtexTER_ENABLE_DOCS=true, docs endpoints become available."""

        # We need a fresh fastapi app created with docs enabled.
        # Re-import create_app after setting the env var so it picks up
        # the patched value at module level.
        import importlib
        import contexter_server.main

        monkeypatch.setenv("CONtexTER_ENABLE_DOCS", "true")
        importlib.reload(contexter_server.main)
        create_app = contexter_server.main.create_app

        app = create_app(data_path="/tmp/contexter-test")
        test_client = TestClient(app, base_url="http://localhost")

        assert test_client.get("/docs").status_code == 200
        assert test_client.get("/openapi.json").status_code == 200


# ---------------------------------------------------------------------------
# 8d: Body Size Limiting Middleware
# ---------------------------------------------------------------------------


class TestBodySizeLimit:
    """8d: Body Size Limiting Middleware."""

    def test_small_request_accepted(self, client: TestClient) -> None:
        """Normal-size request bodies are accepted."""
        resp = client.get("/api/v1/sessions")
        assert resp.status_code < 400

    def test_large_content_length_rejected(self, client: TestClient) -> None:
        """Request with Content-Length exceeding MAX_REQUEST_BODY returns 413."""
        # Default max is 1 MiB (1 048 576 bytes).
        # Cannot actually send 1 MiB, but can set Content-Length header.
        resp = client.get(
            "/api/v1/sessions",
            headers={"Content-Length": "99999999"},
        )
        assert resp.status_code == 413

    def test_chunked_encoding_rejected(self, client: TestClient) -> None:
        """Request with Transfer-Encoding: chunked returns 413."""
        resp = client.get(
            "/api/v1/sessions",
            headers={"Transfer-Encoding": "chunked"},
        )
        assert resp.status_code == 413
        assert resp.json()["detail"] == "Transfer-Encoding chunked not supported"


# ---------------------------------------------------------------------------
# 8e: TrustedHostMiddleware
# ---------------------------------------------------------------------------


class TestTrustedHosts:
    """8e: TrustedHostMiddleware allows localhost and 127.0.0.1."""

    def test_request_with_unknown_host_rejected(self, client: TestClient) -> None:
        """Request from an unknown Host header is rejected."""
        resp = client.get("/health", headers={"host": "evil.com"})
        assert resp.status_code == 400

    def test_request_with_allowed_host_accepted(self, client: TestClient) -> None:
        """Request with localhost Host header works."""
        resp = client.get("/health", headers={"host": "localhost"})
        assert resp.status_code == 200

    def test_request_with_loopback_accepted(self, client: TestClient) -> None:
        """Request with 127.0.0.1 Host header works."""
        resp = client.get("/health", headers={"host": "127.0.0.1"})
        assert resp.status_code == 200


# ---------------------------------------------------------------------------
# 8f: Path Traversal Protection
# ---------------------------------------------------------------------------


class TestPathTraversalProtection:
    """8f: Path Traversal Protection in files API."""

    def test_safe_path_accepted(self, client: TestClient) -> None:
        """Normal path within base_dir is accepted."""
        resp = client.get("/api/v1/files?path=.")
        assert resp.status_code == 200
        assert "files" in resp.json()

    def test_path_outside_base_dir_rejected(self, client: TestClient) -> None:
        """Path outside the configured base directory returns 403."""
        resp = client.get("/api/v1/files?path=/tmp")
        assert resp.status_code == 403

    def test_path_with_dotdot_rejected(self, client: TestClient) -> None:
        """Path containing .. is rejected as invalid (400)."""
        resp = client.get("/api/v1/files", params={"path": "../../etc/passwd"})
        assert resp.status_code == 400

    def test_path_traversal_with_encoded_dotdot(self, client: TestClient) -> None:
        """URL-encoded path traversal is also rejected.

        Uses a raw URL string to avoid httpx double-encoding the
        percent-encoded sequences.
        """
        # Use raw URL to prevent httpx from re-encoding %2F → %252F
        resp = client.get(
            "/api/v1/files?path=..%2F..%2Fetc%2Fpasswd",
        )
        assert resp.status_code == 400


# ---------------------------------------------------------------------------
# 8g: Debug Mode
# ---------------------------------------------------------------------------


class TestDebugMode:
    """8g: Debug mode is explicitly disabled."""

    def test_debug_is_false(self, client: TestClient, app: "FastAPI") -> None:
        """FastAPI debug mode is set to False."""
        assert app.debug is False


# ---------------------------------------------------------------------------
# Validation of validate_safe_path directly
# ---------------------------------------------------------------------------


class TestValidateSafePath:
    """Unit tests for validate_safe_path function (8f)."""

    def test_resolves_and_returns_path(self) -> None:
        """validate_safe_path resolves a path correctly."""
        from contexter_server.api.files import validate_safe_path

        result = validate_safe_path("/tmp")
        assert str(result) == "/tmp"

    def test_rejects_dotdot(self) -> None:
        """Path with .. is rejected."""
        from contexter_server.api.files import validate_safe_path

        with pytest.raises(Exception):
            validate_safe_path("../../etc/passwd")

    def test_path_within_base_dir_accepted(self) -> None:
        """Path inside base_dir passes the confinement check."""
        from contexter_server.api.files import validate_safe_path

        result = validate_safe_path("/tmp/foo/bar", base_dir="/tmp")
        assert str(result) == "/tmp/foo/bar"

    def test_path_exactly_base_dir_accepted(self) -> None:
        """Path equal to base_dir passes."""
        from contexter_server.api.files import validate_safe_path

        result = validate_safe_path("/tmp", base_dir="/tmp")
        assert str(result) == "/tmp"

    def test_path_outside_base_dir_rejected(self) -> None:
        """Path outside base_dir returns 403."""
        from contexter_server.api.files import validate_safe_path

        with pytest.raises(Exception) as exc:
            validate_safe_path("/etc/passwd", base_dir="/tmp")
        assert exc.value.status_code == 403

    def test_base_dir_prefix_not_confused(self) -> None:
        """Path like /tmp2 is not considered inside /tmp."""
        from contexter_server.api.files import validate_safe_path

        with pytest.raises(Exception) as exc:
            validate_safe_path("/tmp2/foo", base_dir="/tmp")
        assert exc.value.status_code == 403

    def test_base_dir_none_skips_confinement(self) -> None:
        """When base_dir is None, no confinement check runs."""
        from contexter_server.api.files import validate_safe_path

        # Should not raise despite being outside default cwd
        result = validate_safe_path("/tmp/foo")
        assert str(result) == "/tmp/foo"
