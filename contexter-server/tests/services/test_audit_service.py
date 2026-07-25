"""Tests for AuditService."""

from unittest.mock import AsyncMock

import pytest

from contexter_server.models.audit import AuditEntry, AuditFilter
from contexter_server.services.audit_service import AuditService


@pytest.fixture
def mock_engine():
    engine = AsyncMock()
    return engine


@pytest.fixture
def service(mock_engine):
    return AuditService(mock_engine)


class TestAuditServiceQuery:
    """Tests for AuditService.query."""

    @pytest.mark.asyncio
    async def test_queries_audit_entries(self, service, mock_engine, any_uuid):
        mock_engine.query_audit.return_value = [
            {
                "id": any_uuid,
                "entity_type": "session",
                "entity_id": any_uuid,
                "action": "created",
                "actor": "user-1",
                "details": {},
            },
        ]
        filter_data = AuditFilter(entity_type="session", limit=10)
        result = await service.query(filter_data)
        assert len(result) == 1
        assert result[0].entity_type == "session"
        assert result[0].action == "created"

    @pytest.mark.asyncio
    async def test_returns_empty_when_no_matches(self, service, mock_engine):
        mock_engine.query_audit.return_value = []
        filter_data = AuditFilter(entity_type="nonexistent")
        result = await service.query(filter_data)
        assert result == []


class TestAuditServiceLog:
    """Tests for AuditService.log."""

    @pytest.mark.asyncio
    async def test_logs_audit_entry(self, service, mock_engine):
        await service.log(
            entity_type="session",
            entity_id="sid-1",
            action="created",
            actor="user-1",
            details={"reason": "test"},
        )
        mock_engine.log_audit.assert_awaited_once()
        # Verify the entry dict was passed with correct fields
        call_args = mock_engine.log_audit.await_args[0][0]
        assert call_args["entity_type"] == "session"
        assert call_args["entity_id"] == "sid-1"
        assert call_args["action"] == "created"
        assert call_args["actor"] == "user-1"

    @pytest.mark.asyncio
    async def test_logs_without_actor_and_details(self, service, mock_engine):
        await service.log(
            entity_type="memory",
            entity_id="mem-1",
            action="deleted",
        )
        mock_engine.log_audit.assert_awaited_once()
        call_args = mock_engine.log_audit.await_args[0][0]
        assert call_args["entity_type"] == "memory"
        assert call_args["actor"] is None
        assert call_args["details"] == {}
