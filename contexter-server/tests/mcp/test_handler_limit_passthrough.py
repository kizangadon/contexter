"""Handler limit passthrough tests — Bug 2026-08-01-handler-limit-passthrough.

Proves ``handle_list_recent_sessions`` pushes the clamped limit into
``session_service.list()`` so the ENGINE honors it (REQ-HLP-001), never
re-slices the service result (REQ-HLP-002), and preserves absent-limit
behaviour (REQ-HLP-003). The MCP tool signature and ``SessionFilter``
shape are unchanged (REQ-HLP-004).

These tests fail on the unfixed code (handler calls ``list(filter=...)``
without ``limit`` and re-slices in Python) and pass after the fix.
"""

from unittest.mock import AsyncMock
from uuid import UUID

import pytest

from contexter_server.mcp_tools.handlers import handle_list_recent_sessions
from contexter_server.models.session import Session
from contexter_server.services.session_service import MAX_SESSION_LIST_LIMIT


@pytest.fixture
def session_service() -> AsyncMock:
    """Mock service at the handler boundary (spy for the engine call)."""
    return AsyncMock()


def _session(project: str = "test-project") -> Session:
    return Session(
        id=UUID("00000000-0000-0000-0000-000000000001"),
        agent_id=UUID("00000000-0000-0000-0000-000000000001"),
        project=project,
    )


class TestLimitReachesService:
    @pytest.mark.asyncio
    async def test_explicit_limit_is_forwarded(self, session_service):
        """REQ-HLP-001/AC-HLP-001: limit=5 reaches the service as 5."""
        session_service.list.return_value = [_session()] * 5

        result = await handle_list_recent_sessions(
            limit=5,
            session_service=session_service,
        )

        session_service.list.assert_awaited_once_with(filter=None, limit=5)
        assert len(result["sessions"]) == 5

    @pytest.mark.asyncio
    async def test_absent_limit_passes_none(self, session_service):
        """REQ-HLP-003/AC-HLP-002: limit=None reaches the service as None
        so the engine default (100) applies."""
        session_service.list.return_value = [_session(), _session()]

        result = await handle_list_recent_sessions(session_service=session_service)

        session_service.list.assert_awaited_once_with(filter=None, limit=None)
        assert len(result["sessions"]) == 2

    @pytest.mark.asyncio
    async def test_negative_limit_is_clamped_to_zero(self, session_service):
        """AC-HLP-003/EC-HLP-003: limit=-1 reaches the service clamped to 0."""
        session_service.list.return_value = []

        result = await handle_list_recent_sessions(
            limit=-1,
            session_service=session_service,
        )

        session_service.list.assert_awaited_once_with(filter=None, limit=0)
        assert result["sessions"] == []

    @pytest.mark.asyncio
    async def test_zero_limit_is_clamped_to_zero(self, session_service):
        """AC-HLP-003/EC-HLP-002: limit=0 reaches the service as 0."""
        session_service.list.return_value = []

        result = await handle_list_recent_sessions(
            limit=0,
            session_service=session_service,
        )

        session_service.list.assert_awaited_once_with(filter=None, limit=0)
        assert result["sessions"] == []

    @pytest.mark.asyncio
    async def test_huge_limit_is_clamped_to_max(self, session_service):
        """AC-HLP-004/EC-HLP-004: limit=10**9 reaches the service as
        MAX_SESSION_LIST_LIMIT — no unbounded fetch."""
        session_service.list.return_value = [_session()]

        result = await handle_list_recent_sessions(
            limit=10**9,
            session_service=session_service,
        )

        session_service.list.assert_awaited_once_with(
            filter=None, limit=MAX_SESSION_LIST_LIMIT
        )
        assert len(result["sessions"]) == 1


class TestServiceResultAuthoritative:
    @pytest.mark.asyncio
    async def test_no_python_reslice(self, session_service):
        """REQ-HLP-002/AC-HLP-005: the service result is authoritative —
        no truncation after the service call."""
        sessions = [_session() for _ in range(5)]
        session_service.list.return_value = sessions

        result = await handle_list_recent_sessions(
            limit=2,
            session_service=session_service,
        )

        session_service.list.assert_awaited_once_with(filter=None, limit=2)
        assert len(result["sessions"]) == 5  # engine is authoritative

    @pytest.mark.asyncio
    async def test_engine_fewer_than_limit_returned_unchanged(self, session_service):
        """EC-HLP-006: engine returns fewer than limit → response returns
        exactly what the engine returned."""
        session_service.list.return_value = [_session()]

        result = await handle_list_recent_sessions(
            limit=50,
            session_service=session_service,
        )

        assert len(result["sessions"]) == 1


class TestFilterShapeUnchanged:
    @pytest.mark.asyncio
    async def test_project_filter_still_passed(self, session_service):
        """REQ-HLP-004: SessionFilter shape unchanged when limit is passed."""
        session_service.list.return_value = []

        await handle_list_recent_sessions(
            limit=5,
            project="my-project",
            session_service=session_service,
        )

        call_kwargs = session_service.list.call_args[1]
        assert call_kwargs["filter"] is not None
        assert call_kwargs["filter"].project == "my-project"
        assert call_kwargs["limit"] == 5

    @pytest.mark.asyncio
    async def test_no_project_passes_none_filter(self, session_service):
        """REQ-HLP-004: no project → filter=None (frozen shape)."""
        session_service.list.return_value = []

        await handle_list_recent_sessions(session_service=session_service)

        session_service.list.assert_awaited_once_with(filter=None, limit=None)
