"""RED reproduction tests — Bug 2026-08-01-input-validation-gaps.

Frozen contract requires:
- store_memory rejects empty/whitespace-only content (EC-006)
- export_data rejects unsupported format (EC-012)
- limit clamped to sane bounds, never crashes (EC-009 / EC-IV-005..007)
- size caps on content/query with structured errors, no unbounded echo
  (REQ-IV-004 / REQ-IV-005, EC-IV-008..009)

These tests fail on the unfixed code and pass after the validation fix.
"""

from unittest.mock import AsyncMock
from uuid import UUID

import pytest

from contexter_server.mcp_server import create_mcp_server
from contexter_server.mcp_tools.errors import HandlerError
from contexter_server.mcp_tools.handlers import (
    handle_export_data,
    handle_list_recent_sessions,
    handle_search_memories,
    handle_store_memory,
)
from contexter_server.models.search import SearchResponse, SearchResult
from contexter_server.models.session import Session


@pytest.fixture
def mock_services():
    return {
        "memory_service": AsyncMock(),
        "session_service": AsyncMock(),
        "agent_service": AsyncMock(),
        "skill_service": AsyncMock(),
        "analytics_service": AsyncMock(),
        "export_service": AsyncMock(),
    }


SID = "00000000-0000-0000-0000-000000000001"


def _session() -> Session:
    return Session(
        id=UUID(SID),
        agent_id=UUID(SID),
        project="test-project",
    )


# ── REQ-IV-001: empty content rejected ───────────────────────────────────


class TestEmptyContent:
    @pytest.mark.asyncio
    async def test_store_memory_rejects_empty_content(self, mock_services):
        """content="" → structured error, nothing persisted."""
        mock_services["session_service"].get.return_value = _session()

        with pytest.raises(HandlerError) as exc:
            await handle_store_memory(
                session_id=SID,
                role="user",
                content="",
                memory_service=mock_services["memory_service"],
                session_service=mock_services["session_service"],
            )
        assert "content" in str(exc.value).lower()
        mock_services["memory_service"].create.assert_not_awaited()

    @pytest.mark.asyncio
    async def test_store_memory_rejects_whitespace_only_content(self, mock_services):
        """content='   ' → structured error (EC-IV-002)."""
        mock_services["session_service"].get.return_value = _session()

        with pytest.raises(HandlerError):
            await handle_store_memory(
                session_id=SID,
                role="user",
                content="   \t  ",
                memory_service=mock_services["memory_service"],
                session_service=mock_services["session_service"],
            )
        mock_services["memory_service"].create.assert_not_awaited()


# ── REQ-IV-002: export format allowlist ──────────────────────────────────


class TestExportFormatAllowlist:
    @pytest.mark.asyncio
    async def test_export_data_rejects_unsupported_format(self, mock_services):
        """format='xml' → structured error, never 'completed' (EC-IV-003)."""
        with pytest.raises(HandlerError) as exc:
            await handle_export_data(
                format="xml",
                export_service=mock_services["export_service"],
            )
        assert "xml" in str(exc.value).lower()
        mock_services["export_service"].submit.assert_not_awaited()

    @pytest.mark.asyncio
    async def test_export_data_accepts_supported_format(self, mock_services):
        """format='json' → success (EC-IV-004)."""
        from contexter_server.models.export import ExportStatus

        mock_services["export_service"].submit.return_value = ExportStatus(
            id=UUID(SID),
            status="completed",
        )

        result = await handle_export_data(
            format="json",
            export_service=mock_services["export_service"],
        )
        assert "error" not in result
        assert result["status"] == "completed"

    @pytest.mark.asyncio
    async def test_export_data_defaults_to_json(self, mock_services):
        """format=None → json (frozen default)."""
        from contexter_server.models.export import ExportStatus

        mock_services["export_service"].submit.return_value = ExportStatus(
            id=UUID(SID),
            status="completed",
        )

        await handle_export_data(export_service=mock_services["export_service"])
        request = mock_services["export_service"].submit.call_args[0][0]
        assert request.format == "json"


# ── REQ-IV-003: limit clamping ───────────────────────────────────────────


class TestLimitClamping:
    @pytest.mark.asyncio
    async def test_list_recent_sessions_clamps_negative_limit(self, mock_services):
        """limit=-5 → service receives clamped 0, never negative (EC-IV-006)."""
        sessions = [_session()]

        async def limit_aware_list(filter=None, limit=None):
            return sessions if limit is None else sessions[:limit]

        mock_services["session_service"].list.side_effect = limit_aware_list

        result = await handle_list_recent_sessions(
            limit=-5,
            session_service=mock_services["session_service"],
        )
        assert "error" not in result
        mock_services["session_service"].list.assert_awaited_once_with(
            filter=None, limit=0
        )
        assert len(result["sessions"]) == 0  # clamped to 0 at the service boundary

    @pytest.mark.asyncio
    async def test_list_recent_sessions_clamps_zero_limit(self, mock_services):
        """limit=0 → clamped, success (EC-IV-005)."""
        mock_services["session_service"].list.return_value = [_session()]

        result = await handle_list_recent_sessions(
            limit=0,
            session_service=mock_services["session_service"],
        )
        assert "error" not in result

    @pytest.mark.asyncio
    async def test_list_recent_sessions_clamps_huge_limit(self, mock_services):
        """limit=10**9 → clamped to max, no crash (EC-IV-007)."""
        mock_services["session_service"].list.return_value = [_session()]

        result = await handle_list_recent_sessions(
            limit=10**9,
            session_service=mock_services["session_service"],
        )
        assert "error" not in result
        assert len(result["sessions"]) == 1

    @pytest.mark.asyncio
    async def test_search_memories_clamps_negative_limit(self, mock_services):
        """search limit=-1 → clamped to a sane value, success."""
        mock_services["memory_service"].search.return_value = SearchResponse()

        result = await handle_search_memories(
            query="test",
            limit=-1,
            memory_service=mock_services["memory_service"],
        )
        assert "error" not in result

    @pytest.mark.asyncio
    async def test_search_memories_clamps_huge_limit(self, mock_services):
        """search limit=10**9 → clamped to SearchQuery max (100)."""
        mock_services["memory_service"].search.return_value = SearchResponse()

        result = await handle_search_memories(
            query="test",
            limit=10**9,
            memory_service=mock_services["memory_service"],
        )
        assert "error" not in result
        call_kwargs = mock_services["memory_service"].search.call_args[0][0]
        assert call_kwargs.limit <= 100


# ── REQ-IV-004/005: size caps, no unbounded echo ─────────────────────────


class TestSizeCaps:
    @pytest.mark.asyncio
    async def test_store_memory_rejects_oversized_content(self, mock_services):
        """content beyond cap → structured error (EC-IV-008)."""
        mock_services["session_service"].get.return_value = _session()
        oversized = "x" * (1_000_000 + 1)

        with pytest.raises(HandlerError) as exc:
            await handle_store_memory(
                session_id=SID,
                role="user",
                content=oversized,
                memory_service=mock_services["memory_service"],
                session_service=mock_services["session_service"],
            )
        message = str(exc.value)
        assert oversized not in message, "error must not echo unbounded input"
        assert "content" in message.lower()
        mock_services["memory_service"].create.assert_not_awaited()

    @pytest.mark.asyncio
    async def test_search_memories_rejects_oversized_query(self, mock_services):
        """query beyond cap → structured error, no echo (EC-IV-009)."""
        oversized = "q" * (10_000 + 1)

        with pytest.raises(HandlerError) as exc:
            await handle_search_memories(
                query=oversized,
                memory_service=mock_services["memory_service"],
            )
        message = str(exc.value)
        assert oversized not in message, "error must not echo unbounded input"
        assert "query" in message.lower()
        mock_services["memory_service"].search.assert_not_awaited()

    @pytest.mark.asyncio
    async def test_search_memories_empty_query_rejected(self, mock_services):
        """query='' → structured validation error."""
        with pytest.raises(HandlerError):
            await handle_search_memories(
                query="   ",
                memory_service=mock_services["memory_service"],
            )
        mock_services["memory_service"].search.assert_not_awaited()


# ── Live protocol: structured error frames for validation failures ───────


class TestLiveValidationFrames:
    @pytest.mark.asyncio
    async def test_live_store_memory_empty_content_is_error_frame(self, mock_services):
        """AC-IV-001 live: empty content → isError frame, nothing persisted."""
        mcp = create_mcp_server(**mock_services)
        mock_services["session_service"].get.return_value = _session()

        from fastmcp import Client

        async with Client(mcp) as client:
            result = await client.call_tool_mcp(
                "store_memory",
                {
                    "session_id": SID,
                    "role": "user",
                    "content": "",
                },
            )

        assert result.isError is True
        mock_services["memory_service"].create.assert_not_awaited()
