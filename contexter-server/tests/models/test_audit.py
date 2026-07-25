"""Tests for audit Pydantic models."""

import uuid
from datetime import datetime

from contexter_server.models.audit import AuditEntry, AuditFilter


class TestAuditModels:
    """Audit model validation tests."""

    def test_audit_entry_defaults(self):
        """AuditEntry should auto-generate id and timestamp."""
        entry = AuditEntry(
            entity_type="session",
            entity_id="abc-123",
            action="created",
        )
        assert isinstance(entry.id, uuid.UUID)
        assert isinstance(entry.timestamp, datetime)
        assert entry.actor is None
        assert entry.details == {}
        assert entry.previous_value is None
        assert entry.new_value is None

    def test_audit_entry_with_all_fields(self):
        """AuditEntry with all fields."""
        entry = AuditEntry(
            entity_type="memory",
            entity_id="mem-001",
            action="updated",
            actor="user-1",
            details={"field": "content"},
            previous_value='{"old": true}',
            new_value='{"new": true}',
        )
        assert entry.actor == "user-1"
        assert entry.details == {"field": "content"}
        assert entry.previous_value == '{"old": true}'
        assert entry.new_value == '{"new": true}'

    def test_audit_entry_serialization(self):
        """AuditEntry should serialize and deserialize."""
        entry = AuditEntry(
            entity_type="session",
            entity_id="abc",
            action="deleted",
        )
        data = entry.model_dump()
        restored = AuditEntry.model_validate(data)
        assert restored.id == entry.id
        assert restored.action == "deleted"

    def test_audit_filter_defaults(self):
        """AuditFilter defaults."""
        f = AuditFilter()
        assert f.entity_type is None
        assert f.action is None
        assert f.actor is None
        assert f.query is None
        assert f.limit == 50
        assert f.offset == 0

    def test_audit_filter_custom(self):
        """AuditFilter with custom values."""
        f = AuditFilter(
            entity_type="session",
            action="created",
            actor="user-1",
            query="test",
            limit=100,
            offset=10,
        )
        assert f.entity_type == "session"
        assert f.action == "created"
        assert f.limit == 100
        assert f.offset == 10

    def test_audit_filter_limit_range(self):
        """AuditFilter limit must be 1-500."""
        from pydantic import ValidationError
        import pytest

        with pytest.raises(ValidationError):
            AuditFilter(limit=0)
        with pytest.raises(ValidationError):
            AuditFilter(limit=501)

    def test_audit_filter_offset_ge_0(self):
        """AuditFilter offset must be >= 0."""
        from pydantic import ValidationError
        import pytest

        with pytest.raises(ValidationError):
            AuditFilter(offset=-1)
