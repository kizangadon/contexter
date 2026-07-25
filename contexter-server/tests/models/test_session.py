"""Tests for session Pydantic models."""

import uuid
from datetime import datetime, timezone

import pytest
from pydantic import ValidationError

from contexter_server.models.session import (
    Session,
    SessionCreate,
    SessionPatch,
    SessionFilter,
)


class TestSessionModel:
    """Session model validation and serialization tests."""

    def test_session_defaults(self):
        """Session should auto-generate id and timestamps."""
        session = Session(agent_id=uuid.uuid4(), project="test")
        assert isinstance(session.id, uuid.UUID)
        assert isinstance(session.started_at, datetime)
        assert isinstance(session.updated_at, datetime)
        assert session.status == "active"
        assert session.metadata == {}

    def test_session_minimal(self):
        """Session should be created with only required fields."""
        agent_id = uuid.uuid4()
        session = Session(agent_id=agent_id, project="test-project")
        assert session.agent_id == agent_id
        assert session.project == "test-project"
        assert session.name is None

    def test_session_with_all_fields(self):
        """Session with all fields populated."""
        agent_id = uuid.uuid4()
        session_id = uuid.uuid4()
        now = datetime.now(timezone.utc)
        session = Session(
            id=session_id,
            agent_id=agent_id,
            project="my-project",
            name="My Session",
            status="paused",
            started_at=now,
            updated_at=now,
            completed_at=now,
            metadata={"key": "value"},
        )
        assert session.id == session_id
        assert session.name == "My Session"
        assert session.status == "paused"
        assert session.completed_at == now
        assert session.metadata == {"key": "value"}

    def test_session_project_min_length(self):
        """Project must be at least 1 character."""
        with pytest.raises(ValidationError):
            Session(agent_id=uuid.uuid4(), project="")

    def test_session_project_max_length(self):
        """Project must not exceed 256 characters."""
        with pytest.raises(ValidationError):
            Session(agent_id=uuid.uuid4(), project="x" * 257)

    def test_session_name_max_length(self):
        """Name must not exceed 512 characters."""
        with pytest.raises(ValidationError):
            Session(agent_id=uuid.uuid4(), project="p", name="x" * 513)

    def test_session_serialization_roundtrip(self):
        """Session should serialize and deserialize cleanly."""
        agent_id = uuid.uuid4()
        session = Session(agent_id=agent_id, project="test")
        data = session.model_dump()
        restored = Session.model_validate(data)
        assert restored.id == session.id
        assert restored.agent_id == session.agent_id
        assert restored.project == session.project
        assert restored.status == session.status

    def test_session_json_serialization(self):
        """Session should serialize to JSON and back."""
        agent_id = uuid.uuid4()
        session = Session(agent_id=agent_id, project="test")
        json_str = session.model_dump_json()
        restored = Session.model_validate_json(json_str)
        assert restored.id == session.id
        assert restored.project == "test"


class TestSessionCreateModel:
    """SessionCreate validation tests."""

    def test_session_create_valid(self):
        """SessionCreate should accept valid data."""
        agent_id = uuid.uuid4()
        data = SessionCreate(agent_id=agent_id, project="test")
        assert data.agent_id == agent_id
        assert data.project == "test"
        assert data.status == "active"
        assert data.metadata == {}

    def test_session_create_with_all_fields(self):
        """SessionCreate with all optional fields."""
        agent_id = uuid.uuid4()
        data = SessionCreate(
            agent_id=agent_id,
            project="test",
            name="My Session",
            status="paused",
            metadata={"env": "dev"},
        )
        assert data.name == "My Session"
        assert data.status == "paused"
        assert data.metadata == {"env": "dev"}

    def test_session_create_invalid_project_empty(self):
        """SessionCreate should reject empty project."""
        with pytest.raises(ValidationError):
            SessionCreate(agent_id=uuid.uuid4(), project="")


class TestSessionPatchModel:
    """SessionPatch validation tests."""

    def test_session_patch_empty(self):
        """SessionPatch should allow empty (all optional) patch."""
        patch = SessionPatch()
        assert patch.name is None
        assert patch.status is None
        assert patch.metadata is None

    def test_session_patch_partial(self):
        """SessionPatch with partial fields."""
        patch = SessionPatch(name="Updated", status="completed")
        assert patch.name == "Updated"
        assert patch.status == "completed"
        assert patch.metadata is None

    def test_session_patch_name_max_length(self):
        """SessionPatch name should enforce max_length."""
        with pytest.raises(ValidationError):
            SessionPatch(name="x" * 513)


class TestSessionFilterModel:
    """SessionFilter validation tests."""

    def test_session_filter_empty(self):
        """SessionFilter should allow empty filter."""
        f = SessionFilter()
        assert f.project is None
        assert f.status is None

    def test_session_filter_fields(self):
        """SessionFilter with fields."""
        f = SessionFilter(project="test", status="active")
        assert f.project == "test"
        assert f.status == "active"
