"""Shared test fixtures for FastAPI endpoint tests.

Uses dependency overrides to inject a mock ``StorageEngine`` so that tests
never touch real Rust/wal storage.
"""

from collections.abc import AsyncIterator
from unittest.mock import AsyncMock
from uuid import uuid4

import pytest
from fastapi import FastAPI
from fastapi.testclient import TestClient

from contexter_server.core.bridge import StorageEngine
from contexter_server.main import create_app
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
# Sample data helpers
# ---------------------------------------------------------------------------


@pytest.fixture
def any_uuid() -> str:
    return "00000000-0000-0000-0000-000000000001"


@pytest.fixture
def sample_session(any_uuid: str) -> dict:
    return {
        "id": any_uuid,
        "agent_id": any_uuid,
        "project": "test-project",
        "name": "Test Session",
        "status": "active",
        "started_at": "2026-07-25T10:00:00Z",
        "updated_at": "2026-07-25T10:00:00Z",
        "completed_at": None,
        "metadata": {},
    }


@pytest.fixture
def sample_memory(any_uuid: str) -> dict:
    return {
        "id": any_uuid,
        "session_id": any_uuid,
        "agent_id": any_uuid,
        "role": "user",
        "content": "Hello world",
        "tokens": None,
        "tokenizer": None,
        "model": None,
        "created_at": "2026-07-25T10:00:00Z",
        "metadata": {},
    }


@pytest.fixture
def sample_agent(any_uuid: str) -> dict:
    return {
        "id": any_uuid,
        "name": "test-agent",
        "provider": "openai",
        "model": "gpt-4o",
        "system_prompt": "You are helpful.",
        "temperature": 0.7,
        "max_tokens": 4096,
        "tools": [],
        "metadata": {},
        "created_at": "2026-07-25T10:00:00Z",
        "updated_at": "2026-07-25T10:00:00Z",
    }


@pytest.fixture
def sample_skill(any_uuid: str) -> dict:
    return {
        "id": any_uuid,
        "name": "test-skill",
        "description": "A test skill",
        "type": "memory",
        "version": "1.0.0",
        "parameters": {},
        "enabled": True,
        "created_at": "2026-07-25T10:00:00Z",
        "updated_at": "2026-07-25T10:00:00Z",
    }


# ---------------------------------------------------------------------------
# Mock engine fixture
# ---------------------------------------------------------------------------


@pytest.fixture
def mock_engine() -> AsyncMock:
    """Create a mock StorageEngine with all methods returning empty defaults."""
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
    engine.count_agents.return_value = 0
    engine.update_agent.return_value = {}
    engine.delete_agent.return_value = None

    # Skills
    engine.create_skill.return_value = {}
    engine.get_skill.return_value = None
    engine.list_skills.return_value = []
    engine.count_skills.return_value = 0
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
    # Analytics — shapes mirror the real Rust engine (snake_case
    # cache_telemetry, camelCase storage_size, nested status).
    engine.count_sessions.return_value = 0
    engine.count_memories.return_value = 0
    engine.list_agents.return_value = []
    engine.list_skills.return_value = []
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

    return engine


# ---------------------------------------------------------------------------
# Test client fixture
# ---------------------------------------------------------------------------


@pytest.fixture
def app(mock_engine: AsyncMock) -> FastAPI:
    """Create a test FastAPI app with mock services."""
    app = create_app(data_path="/tmp/contexter-test")

    # Override services with mock-backed instances
    services = {
        "session_service": SessionService(mock_engine),
        "memory_service": MemoryService(mock_engine),
        "agent_service": AgentService(mock_engine),
        "skill_service": SkillService(mock_engine),
        "analytics_service": AnalyticsService(mock_engine),
        "search_service": SearchService(mock_engine),
        "settings_service": SettingsService(mock_engine, config_path="/tmp/contexter-test-config.yaml"),
        "notification_service": NotificationService(mock_engine),
        "audit_service": AuditService(mock_engine),
        "correlation_service": CorrelationService(mock_engine),
        "export_service": ExportService(mock_engine),
        "onboarding_service": OnboardingService(mock_engine),
    }
    for attr, svc in services.items():
        setattr(app.state, attr, svc)
    app.state.storage_engine = mock_engine

    return app


@pytest.fixture
def client(app: FastAPI) -> TestClient:
    """Return a TestClient against the test app.

    Uses ``base_url="http://localhost"`` so that
    ``TrustedHostMiddleware`` (which allows ``localhost`` and
    ``127.0.0.1``) does not reject requests.
    """
    return TestClient(app, base_url="http://localhost")
