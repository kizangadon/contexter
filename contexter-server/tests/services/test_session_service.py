"""Tests for SessionService."""

from unittest.mock import AsyncMock

import pytest

from contexter_server.models.session import Session, SessionCreate, SessionFilter, SessionPatch
from contexter_server.services.session_service import MAX_SESSION_LIST_LIMIT, SessionService


@pytest.fixture
def mock_engine():
    engine = AsyncMock()
    return engine


@pytest.fixture
def service(mock_engine):
    return SessionService(mock_engine)


class TestSessionServiceCreate:
    """Tests for SessionService.create."""

    @pytest.mark.asyncio
    async def test_creates_session(self, service, mock_engine, any_uuid):
        sid = any_uuid.replace("1", "2")
        mock_engine.create_session.return_value = {
            "id": sid,
            "agent_id": any_uuid,
            "project": "test-project",
            "name": "Test Session",
            "status": "active",
        }
        data = SessionCreate(agent_id=any_uuid, project="test-project", name="Test Session")
        result = await service.create(data)
        assert str(result.id) == sid
        assert result.project == "test-project"
        assert result.status == "active"
        mock_engine.create_session.assert_awaited_once()


class TestSessionServiceGet:
    """Tests for SessionService.get."""

    @pytest.mark.asyncio
    async def test_gets_session(self, service, mock_engine, any_uuid):
        mock_engine.get_session.return_value = {
            "id": any_uuid,
            "agent_id": any_uuid,
            "project": "test-project",
            "status": "active",
        }
        result = await service.get(any_uuid)
        assert result is not None
        assert str(result.id) == any_uuid
        mock_engine.get_session.assert_awaited_once_with(any_uuid)

    @pytest.mark.asyncio
    async def test_get_returns_none_when_not_found(self, service, mock_engine):
        mock_engine.get_session.return_value = None
        result = await service.get("nonexistent")
        assert result is None


class TestSessionServiceList:
    """Tests for SessionService.list."""

    @pytest.mark.asyncio
    async def test_lists_sessions(self, service, mock_engine, any_uuid):
        sid1 = any_uuid.replace("000001", "000002")
        sid2 = any_uuid.replace("000001", "000003")
        mock_engine.list_sessions.return_value = [
            {"id": sid1, "agent_id": any_uuid, "project": "p1", "status": "active"},
            {"id": sid2, "agent_id": any_uuid, "project": "p2", "status": "completed"},
        ]
        result = await service.list()
        assert len(result) == 2
        assert result[0].project == "p1"
        assert result[1].status == "completed"

    @pytest.mark.asyncio
    async def test_lists_with_filter(self, service, mock_engine, any_uuid):
        mock_engine.list_sessions.return_value = [
            {"id": any_uuid, "agent_id": any_uuid, "project": "test", "status": "active"},
        ]
        filter_data = SessionFilter(project="test")
        result = await service.list(filter_data)
        assert len(result) == 1
        mock_engine.list_sessions.assert_awaited_once()

    @pytest.mark.asyncio
    async def test_list_returns_empty(self, service, mock_engine):
        mock_engine.list_sessions.return_value = []
        result = await service.list()
        assert result == []


class TestSessionServiceListLimitPushdown:
    """Limit pushdown + clamping (bug 2026-08-01-session-limit-pushdown)."""

    @pytest.mark.asyncio
    async def test_list_pushes_limit_to_engine(self, service, mock_engine, any_uuid):
        """REQ-SL-004: list(limit=5) must forward the limit to the engine."""
        mock_engine.list_sessions.return_value = [
            {"id": any_uuid, "agent_id": any_uuid, "project": "p1", "status": "active"},
        ]
        result = await service.list(limit=5)
        assert len(result) == 1
        mock_engine.list_sessions.assert_awaited_once_with(None, limit=5)

    @pytest.mark.asyncio
    async def test_list_pushes_limit_with_filter(self, service, mock_engine, any_uuid):
        """The limit must be forwarded together with the session filter."""
        mock_engine.list_sessions.return_value = [
            {"id": any_uuid, "agent_id": any_uuid, "project": "test", "status": "active"},
        ]
        filter_data = SessionFilter(project="test")
        result = await service.list(filter_data, limit=2)
        assert len(result) == 1
        mock_engine.list_sessions.assert_awaited_once_with({"project": "test"}, limit=2)

    @pytest.mark.asyncio
    async def test_list_clamps_negative_limit_to_zero(self, service, mock_engine):
        """EC-SL-004: a negative limit is clamped to 0, never raised."""
        mock_engine.list_sessions.return_value = []
        result = await service.list(limit=-1)
        assert result == []
        mock_engine.list_sessions.assert_awaited_once_with(None, limit=0)

    @pytest.mark.asyncio
    async def test_list_accepts_zero_limit(self, service, mock_engine):
        """EC-SL-003: limit=0 is valid and returns no sessions."""
        mock_engine.list_sessions.return_value = []
        result = await service.list(limit=0)
        assert result == []
        mock_engine.list_sessions.assert_awaited_once_with(None, limit=0)

    @pytest.mark.asyncio
    async def test_list_clamps_huge_limit_to_max(self, service, mock_engine, any_uuid):
        """EC-SL-005/AC-SL-002: limits above the documented max clamp to it."""
        mock_engine.list_sessions.return_value = [
            {"id": any_uuid, "agent_id": any_uuid, "project": "p1", "status": "active"},
        ]
        result = await service.list(limit=1_000_000)
        assert len(result) == 1
        mock_engine.list_sessions.assert_awaited_once_with(None, limit=MAX_SESSION_LIST_LIMIT)

    @pytest.mark.asyncio
    async def test_list_without_limit_keeps_engine_default(self, service, mock_engine, any_uuid):
        """REQ-SL-001 regression: no limit means the engine default page size."""
        mock_engine.list_sessions.return_value = [
            {"id": any_uuid, "agent_id": any_uuid, "project": "p1", "status": "active"},
        ]
        result = await service.list()
        assert len(result) == 1
        mock_engine.list_sessions.assert_awaited_once_with(None)


class TestSessionServiceUpdate:
    """Tests for SessionService.update."""

    @pytest.mark.asyncio
    async def test_updates_session(self, service, mock_engine, any_uuid):
        mock_engine.update_session.return_value = {
            "id": any_uuid,
            "agent_id": any_uuid,
            "project": "test-project",
            "name": "Updated",
            "status": "paused",
        }
        patch = SessionPatch(name="Updated", status="paused")
        result = await service.update(any_uuid, patch)
        assert result is not None
        assert result.name == "Updated"
        assert result.status == "paused"

    @pytest.mark.asyncio
    async def test_update_returns_none_when_empty(self, service, mock_engine):
        mock_engine.update_session.return_value = {}
        patch = SessionPatch(name="Updated")
        result = await service.update("nonexistent", patch)
        assert result is None


class TestSessionServiceDelete:
    """Tests for SessionService.delete."""

    @pytest.mark.asyncio
    async def test_deletes_session(self, service, mock_engine, any_uuid):
        await service.delete(any_uuid)
        mock_engine.delete_session.assert_awaited_once_with(any_uuid)


class TestSessionServiceResume:
    """Tests for SessionService.resume."""

    @pytest.mark.asyncio
    async def test_resumes_session(self, service, mock_engine, any_uuid):
        mock_engine.update_session.return_value = {
            "id": any_uuid,
            "agent_id": any_uuid,
            "project": "test",
            "status": "active",
        }
        result = await service.resume(any_uuid)
        assert result is not None
        assert result.status == "active"
        mock_engine.update_session.assert_awaited_once()

    @pytest.mark.asyncio
    async def test_resume_returns_none_when_not_found(self, service, mock_engine):
        mock_engine.update_session.return_value = {}
        result = await service.resume("nonexistent")
        assert result is None


class TestSessionServiceComputeEfficiency:
    """Tests for SessionService.compute_efficiency."""

    @pytest.mark.asyncio
    async def test_returns_default_efficiency(self, service, mock_engine, any_uuid):
        mock_engine.get_session.return_value = {
            "id": any_uuid,
            "agent_id": any_uuid,
            "project": "test",
            "status": "active",
        }
        result = await service.compute_efficiency(any_uuid)
        assert result == 1.0

    @pytest.mark.asyncio
    async def test_returns_zero_for_missing_session(self, service, mock_engine):
        mock_engine.get_session.return_value = None
        result = await service.compute_efficiency("nonexistent")
        assert result == 0.0
