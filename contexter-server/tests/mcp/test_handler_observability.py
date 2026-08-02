"""RED reproduction tests — Bug 2026-08-01-handler-observability.

CON-003 requires handler-level structured logs: call received (tool,
session id, correlation id), auth decision, engine result (success/error,
duration), error path — with no content payloads or secrets (B9 bounds).

These tests fail on the unfixed code (no handler logging) and pass after
structured logging is added.
"""

import os
import re
from unittest import mock
from unittest.mock import AsyncMock
from uuid import UUID

import pytest

from contexter_server.mcp_tools.handlers import (
    handle_get_session,
    handle_store_memory,
)
from contexter_server.models.memory import Memory
from contexter_server.models.session import Session


_ANSI_ESCAPE = re.compile(r"\x1b\[[0-9;]*m")


def _plain_message(record) -> str:
    """Strip ANSI colour codes from structlog's ConsoleRenderer output."""
    return _ANSI_ESCAPE.sub("", record.getMessage())


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


class TestHandlerLogsSuccessPath:
    @pytest.mark.asyncio
    async def test_success_path_emits_call_auth_result_logs(self, mock_services, caplog):
        """AC-HO-001: call-received, auth-decision, engine-result logs exist."""
        session = Session(
            id=UUID(SID),
            agent_id=UUID(SID),
            project="test-project",
            name="Test Session",
        )
        memory = Memory(
            id=UUID(SID),
            session_id=UUID(SID),
            agent_id=UUID(SID),
            role="user",
            content="Hello, world!",
        )
        mock_services["session_service"].get.return_value = session
        mock_services["memory_service"].create.return_value = memory

        with caplog.at_level("DEBUG"):
            await handle_store_memory(
                session_id=SID,
                role="user",
                content="Hello, world!",
                memory_service=mock_services["memory_service"],
                session_service=mock_services["session_service"],
            )

        events = [_plain_message(r) for r in caplog.records]
        joined = " ".join(events)
        assert any("call_received" in e for e in events), (
            f"missing call_received log; got: {joined}"
        )
        assert any("auth_decision" in e for e in events), (
            f"missing auth_decision log; got: {joined}"
        )
        assert any("engine_result" in e for e in events), (
            f"missing engine_result log; got: {joined}"
        )

    @pytest.mark.asyncio
    async def test_success_path_logs_do_not_leak_content(self, mock_services, caplog):
        """REQ-HO-002: no content payloads in logs."""
        session = Session(
            id=UUID(SID),
            agent_id=UUID(SID),
            project="test-project",
        )
        memory = Memory(
            id=UUID(SID),
            session_id=UUID(SID),
            agent_id=UUID(SID),
            role="user",
            content="SUPER_SECRET_PAYLOAD_XYZ",
        )
        mock_services["session_service"].get.return_value = session
        mock_services["memory_service"].create.return_value = memory

        with caplog.at_level("DEBUG"):
            await handle_store_memory(
                session_id=SID,
                role="user",
                content="SUPER_SECRET_PAYLOAD_XYZ",
                memory_service=mock_services["memory_service"],
                session_service=mock_services["session_service"],
            )

        joined = " ".join(_plain_message(r) for r in caplog.records)
        assert "SUPER_SECRET_PAYLOAD_XYZ" not in joined, "content leaked into logs"


class TestHandlerLogsErrorPath:
    @pytest.mark.asyncio
    async def test_not_found_emits_error_log_with_correlation_id(
        self, mock_services, caplog
    ):
        """AC-HO-002: error path emits a structured error log."""
        mock_services["session_service"].get.return_value = None

        from contexter_server.mcp_tools.errors import HandlerError

        with caplog.at_level("INFO"):
            with pytest.raises(HandlerError):
                await handle_get_session(
                    id="missing-session-1",
                    session_service=mock_services["session_service"],
                )

        events = [_plain_message(r) for r in caplog.records]
        joined = " ".join(events)
        assert any("error" in e.lower() or "not_found" in e.lower() for e in events), (
            f"missing error log; got: {joined}"
        )
        # Correlation id must flow through the log lines
        assert "correlation_id" in joined or "corr" in joined.lower(), (
            f"no correlation id in logs; got: {joined}"
        )

    @pytest.mark.asyncio
    async def test_auth_reject_logs_contain_no_secrets(self, mock_services, caplog):
        """EC-HO-001: auth reject logs without leaking the key."""
        with mock.patch.dict(os.environ, {"CONTEXTER_API_KEY": "top-secret-key"}):
            with caplog.at_level("INFO"):
                from contexter_server.mcp_tools.auth import MCPAuthError

                with pytest.raises(MCPAuthError):
                    await handle_get_session(
                        id=SID,
                        session_service=mock_services["session_service"],
                    )

        joined = " ".join(_plain_message(r) for r in caplog.records)
        assert "top-secret-key" not in joined, "API key leaked into logs"


class TestCorrelationIds:
    @pytest.mark.asyncio
    async def test_concurrent_calls_have_distinct_correlation_ids(
        self, mock_services, caplog
    ):
        """EC-HO-004: concurrent handler calls must produce distinct corr ids."""
        import asyncio

        session = Session(
            id=UUID(SID),
            agent_id=UUID(SID),
            project="test-project",
            name="Test Session",
        )
        mock_services["session_service"].get.return_value = session

        with caplog.at_level("DEBUG"):
            await asyncio.gather(
                handle_get_session(
                    id=SID, session_service=mock_services["session_service"]
                ),
                handle_get_session(
                    id=SID, session_service=mock_services["session_service"]
                ),
            )

        ids = set()
        for record in caplog.records:
            m = re.search(r"correlation_id[=: ]+([0-9a-f]{32})", _plain_message(record))
            if m:
                ids.add(m.group(1))
        assert len(ids) >= 2, (
            f"expected distinct correlation ids per call, got {len(ids)}"
        )


class TestPerCallRequestLogLevels:
    """REQ-PLB-001 / PF-05: per-call request logs are DEBUG, not INFO.

    INFO is reserved for lifecycle and error events; every per-request log
    (call received, auth decision, engine result) must be emitted at DEBUG so
    INFO-level sinks do not pay per-call logging cost.
    """

    @pytest.mark.asyncio
    async def test_per_call_request_logs_are_emitted_at_debug(
        self, mock_services, caplog
    ):
        """AC-PLB-001: call_received/auth_decision/engine_result use DEBUG."""
        session = Session(
            id=UUID(SID),
            agent_id=UUID(SID),
            project="test-project",
            name="Test Session",
        )
        memory = Memory(
            id=UUID(SID),
            session_id=UUID(SID),
            agent_id=UUID(SID),
            role="user",
            content="Hello, world!",
        )
        mock_services["session_service"].get.return_value = session
        mock_services["memory_service"].create.return_value = memory

        with caplog.at_level("DEBUG"):
            await handle_store_memory(
                session_id=SID,
                role="user",
                content="Hello, world!",
                memory_service=mock_services["memory_service"],
                session_service=mock_services["session_service"],
            )

        levels = {}
        for record in caplog.records:
            message = _plain_message(record)
            for event in ("call_received", "auth_decision", "engine_result"):
                if event in message:
                    levels[event] = record.levelname

        assert levels == {
            "call_received": "DEBUG",
            "auth_decision": "DEBUG",
            "engine_result": "DEBUG",
        }, f"per-call request logs must be DEBUG; got: {levels}"

    @pytest.mark.asyncio
    async def test_per_call_request_logs_are_absent_at_info(
        self, mock_services, caplog
    ):
        """INFO sinks must not see per-call request events (REQ-PLB-001)."""
        session = Session(
            id=UUID(SID),
            agent_id=UUID(SID),
            project="test-project",
            name="Test Session",
        )
        memory = Memory(
            id=UUID(SID),
            session_id=UUID(SID),
            agent_id=UUID(SID),
            role="user",
            content="Hello, world!",
        )
        mock_services["session_service"].get.return_value = session
        mock_services["memory_service"].create.return_value = memory

        with caplog.at_level("INFO"):
            await handle_store_memory(
                session_id=SID,
                role="user",
                content="Hello, world!",
                memory_service=mock_services["memory_service"],
                session_service=mock_services["session_service"],
            )

        joined = " ".join(_plain_message(r) for r in caplog.records)
        for event in ("call_received", "auth_decision", "engine_result"):
            assert event not in joined, (
                f"{event} must not be logged at INFO level; got: {joined}"
            )
