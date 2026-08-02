"""Tests for rate limiting middleware (BUG-020).

Verifies that:
- Exceeding the configured rate limit returns 429.
- The ``/health`` endpoint is exempt from rate limiting.
- Rate limiting can be disabled via ``CONTEXTER_RATE_LIMIT_ENABLED=false``.
"""

import os
from unittest import mock
from unittest.mock import AsyncMock

import pytest
from fastapi import FastAPI
from fastapi.testclient import TestClient

from contexter_server.main import create_app
from contexter_server.core.bridge import StorageEngine
from contexter_server.services import (
    AgentService,
    AnalyticsService,
    AuditService,
    CorrelationService,
    ExportService,
    MemoryService,
    NotificationService,
    OnboardingService,
    SearchService,
    SessionService,
    SettingsService,
    SkillService,
)

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def _build_app_with_limit(limit: str) -> FastAPI:
    """Create a test ``FastAPI`` app whose default rate limit is *limit*.

    All domain services are backed by a mock ``StorageEngine`` so the
    resulting app is safe to use in unit tests.
    """
    with mock.patch.dict(os.environ, {"CONTEXTER_RATE_LIMIT": limit}):
        app = create_app(data_path="/tmp/contexter-test")

    _inject_mock_services(app)
    return app


def _inject_mock_services(app: FastAPI) -> None:
    """Replace real services on *app* with mock-backed instances."""
    engine = AsyncMock(spec=StorageEngine)

    # Sessions
    engine.create_session.return_value = {}
    engine.get_session.return_value = None
    engine.list_sessions.return_value = []
    engine.update_session.return_value = {}
    engine.delete_session.return_value = None

    # Memories
    engine.create_memory.return_value = {}
    engine.get_memory.return_value = None
    engine.search_memories.return_value = []
    engine.update_memory.return_value = {}
    engine.delete_memory.return_value = None
    engine.count_memories.return_value = 0

    # Agents
    engine.create_agent.return_value = {}
    engine.get_agent.return_value = None
    engine.list_agents.return_value = []
    engine.update_agent.return_value = {}
    engine.delete_agent.return_value = None

    # Skills
    engine.create_skill.return_value = {}
    engine.get_skill.return_value = None
    engine.list_skills.return_value = []
    engine.update_skill.return_value = {}
    engine.delete_skill.return_value = None

    # Settings
    engine.get_setting.return_value = None
    engine.set_setting.return_value = None

    # Audit
    engine.log_audit.return_value = None
    engine.query_audit.return_value = []

    # Maintenance
    engine.flush.return_value = None
    engine.checkpoint.return_value = 0
    # Analytics shapes (unused by these tests, but kept truthful to the real
    # Rust engine: snake_case cache_telemetry, camelCase storage_size).
    engine.storage_size.return_value = {"perCf": {}, "total": 0, "walSize": 0}
    engine.status.return_value = {
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
    engine.cache_telemetry.return_value = {
        "gets": 0,
        "hits": 0,
        "misses": 0,
        "stores": 0,
        "invalidations": 0,
        "total_ops": 0,
        "entries_by_type": {},
    }

    services: dict[str, object] = {
        "session_service": SessionService(engine),
        "memory_service": MemoryService(engine),
        "agent_service": AgentService(engine),
        "skill_service": SkillService(engine),
        "analytics_service": AnalyticsService(engine),
        "search_service": SearchService(engine),
        "settings_service": SettingsService(
            engine, config_path="/tmp/contexter-test-config.yaml"
        ),
        "notification_service": NotificationService(engine),
        "audit_service": AuditService(engine),
        "correlation_service": CorrelationService(engine),
        "export_service": ExportService(engine),
        "onboarding_service": OnboardingService(engine),
    }
    for attr, svc in services.items():
        setattr(app.state, attr, svc)
    app.state.storage_engine = engine


# ---------------------------------------------------------------------------
# Tests
# ---------------------------------------------------------------------------


class TestRateLimitExceeded:
    """BUG-020: Rate limiting returns 429 when the limit is exceeded."""

    def test_sends_429_on_rate_limit_exceeded(self) -> None:
        """After consuming the allowed requests, subsequent calls get 429."""
        app = _build_app_with_limit("2/minute")
        client = TestClient(app, base_url="http://localhost")

        # First two requests — within limit
        resp1 = client.get("/api/v1/sessions")
        assert resp1.status_code == 200, (
            f"first request expected 200, got {resp1.status_code}"
        )

        resp2 = client.get("/api/v1/sessions")
        assert resp2.status_code == 200, (
            f"second request expected 200, got {resp2.status_code}"
        )

        # Third request — over the 2/minute limit
        resp3 = client.get("/api/v1/sessions")
        assert resp3.status_code == 429, (
            f"expected 429 rate limited, got {resp3.status_code}"
        )

    def test_rate_limit_error_contains_detail(self) -> None:
        """The 429 response body includes an error message."""
        app = _build_app_with_limit("1/minute")
        client = TestClient(app, base_url="http://localhost")

        # Use the one allowed request
        client.get("/api/v1/sessions")

        resp = client.get("/api/v1/sessions")
        assert resp.status_code == 429
        data = resp.json()
        assert "error" in data, "429 response should contain 'error' key"
        assert "rate limit" in data["error"].lower(), (
            f"error message should mention rate limit, got: {data['error']}"
        )


class TestRateLimitingDisabled:
    """BUG-020: Rate limiting can be disabled."""

    def test_requests_succeed_when_disabled(self) -> None:
        """When CONTEXTER_RATE_LIMIT_ENABLED=false, no 429 is returned."""
        with mock.patch.dict(
            os.environ,
            {
                "CONTEXTER_RATE_LIMIT": "1/minute",
                "CONTEXTER_RATE_LIMIT_ENABLED": "false",
            },
        ):
            app = create_app(data_path="/tmp/contexter-test")
        _inject_mock_services(app)
        client = TestClient(app, base_url="http://localhost")

        # Many requests should all pass
        for _ in range(5):
            resp = client.get("/api/v1/sessions")
            assert resp.status_code == 200, (
                f"expected 200 when rate limit disabled, got {resp.status_code}"
            )


class TestHealthExemptFromRateLimit:
    """BUG-020: The /health endpoint is exempt from rate limiting."""

    def test_health_always_accessible(self) -> None:
        """/health works even after the API limit has been exhausted."""
        app = _build_app_with_limit("1/minute")
        client = TestClient(app, base_url="http://localhost")

        # Consume the API limit on a normal endpoint
        resp = client.get("/api/v1/sessions")
        assert resp.status_code == 200

        # Now /api/v1/ endpoints should be rate-limited
        resp = client.get("/api/v1/sessions")
        assert resp.status_code == 429

        # But /health should still work
        resp = client.get("/health")
        assert resp.status_code == 200, (
            f"/health should be exempt from rate limiting, got {resp.status_code}"
        )
        assert resp.json() == {"status": "ok"}
