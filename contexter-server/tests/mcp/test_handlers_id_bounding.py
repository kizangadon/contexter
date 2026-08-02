"""RED reproduction tests — Bug 2026-08-01-handlers-id-bounding.

not_found_error() echoed unbounded caller-controlled ids in non-UUID-validated
handlers (1MB id → 1,000,020-char error message, violating REQ-IV-005) and
handler log bindings carried raw unbounded ids (1MB request inflating log
lines, violating REQ-HO-002 / B9 bounds). The existing ``_bounded()`` helper
(64-char cap) must be applied at all not-found error sites and all
request-id log bindings, leaving ids ≤ 64 chars byte-identical.

These tests fail on the unfixed code and pass once bounding is applied.
"""

import re
from unittest.mock import AsyncMock

import pytest

from contexter_server.mcp_tools.errors import HandlerError
from contexter_server.mcp_tools.handlers import (
    handle_agent_resource,
    handle_get_agent_info,
    handle_get_session,
    handle_list_recent_sessions,
    handle_list_skills,
    handle_memory_resource,
    handle_session_resource,
    handle_store_memory,
)

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
HUGE_ID = "x" * 1_000_000


# ── AC-HIB-001: not-found errors bound echoed ids (≤ 256 chars) ──────────


class TestNotFoundErrorBounding:
    @pytest.mark.parametrize(
        ("handler", "service_key"),
        [
            (handle_get_session, "session_service"),
            (handle_get_agent_info, "agent_service"),
            (handle_session_resource, "session_service"),
            (handle_memory_resource, "memory_service"),
            (handle_agent_resource, "agent_service"),
        ],
    )
    @pytest.mark.asyncio
    async def test_not_found_bounds_huge_id(self, handler, service_key, mock_services):
        """AC-HIB-001: 1MB id → error ≤ 256 chars, no raw id echoed."""
        mock_services[service_key].get.return_value = None

        with pytest.raises(HandlerError) as exc:
            await handler(id=HUGE_ID, **{service_key: mock_services[service_key]})

        message = str(exc.value)
        assert len(message) <= 256, f"unbounded error message ({len(message)} chars)"
        assert HUGE_ID not in message, "raw unbounded id echoed into error"
        assert message.startswith("Resource not found: ")
        echoed = message.removeprefix("Resource not found: ")
        assert len(echoed) <= 64

    @pytest.mark.asyncio
    async def test_store_memory_huge_session_id_never_echoed(self, mock_services):
        """1MB session_id fails UUID validation with a bounded message."""
        with pytest.raises(HandlerError) as exc:
            await handle_store_memory(
                session_id=HUGE_ID,
                role="user",
                content="hello",
                memory_service=mock_services["memory_service"],
                session_service=mock_services["session_service"],
            )
        message = str(exc.value)
        assert HUGE_ID not in message
        assert len(message) <= 256


# ── AC-HIB-002: request-id log bindings bounded (≤ 64 chars) ─────────────


class TestLogBindingsBounded:
    @pytest.mark.parametrize(
        ("handler", "service_key", "log_key"),
        [
            (handle_get_session, "session_service", "session_id"),
            (handle_get_agent_info, "agent_service", "agent_id"),
            (handle_session_resource, "session_service", "session_id"),
            (handle_memory_resource, "memory_service", "memory_id"),
            (handle_agent_resource, "agent_service", "agent_id"),
        ],
    )
    @pytest.mark.asyncio
    async def test_call_received_bounds_huge_id(
        self, handler, service_key, log_key, mock_services, caplog
    ):
        """AC-HIB-002: 1MB id → log id field ≤ 64 chars."""
        mock_services[service_key].get.return_value = None

        with caplog.at_level("DEBUG"):
            with pytest.raises(HandlerError):
                await handler(id=HUGE_ID, **{service_key: mock_services[service_key]})

        joined = " ".join(_plain_message(r) for r in caplog.records)
        assert HUGE_ID not in joined, "raw unbounded id leaked into logs"
        match = re.search(rf"{log_key}=(\S+)", joined)
        assert match is not None, f"no {log_key} binding found in logs"
        assert len(match.group(1)) <= 64, (
            f"log {log_key} field unbounded ({len(match.group(1))} chars)"
        )

    @pytest.mark.asyncio
    async def test_list_recent_sessions_bounds_huge_project(
        self, mock_services, caplog
    ):
        """1MB project filter → log project field ≤ 64 chars."""
        mock_services["session_service"].list.return_value = []

        with caplog.at_level("DEBUG"):
            await handle_list_recent_sessions(
                project=HUGE_ID, session_service=mock_services["session_service"]
            )

        joined = " ".join(_plain_message(r) for r in caplog.records)
        assert HUGE_ID not in joined, "raw unbounded project leaked into logs"
        match = re.search(r"project=(\S+)", joined)
        assert match is not None, "no project binding found in logs"
        assert len(match.group(1)) <= 64

    @pytest.mark.asyncio
    async def test_list_skills_bounds_huge_type(self, mock_services, caplog):
        """1MB type filter → log type field ≤ 64 chars."""
        mock_services["skill_service"].list.return_value = []

        with caplog.at_level("DEBUG"):
            await handle_list_skills(
                type=HUGE_ID, skill_service=mock_services["skill_service"]
            )

        joined = " ".join(_plain_message(r) for r in caplog.records)
        assert HUGE_ID not in joined, "raw unbounded type leaked into logs"
        match = re.search(r"type=(\S+)", joined)
        assert match is not None, "no type binding found in logs"
        assert len(match.group(1)) <= 64


# ── AC-HIB-003: ids ≤ 64 chars stay byte-identical ───────────────────────


class TestLegitimateIdsUnchanged:
    @pytest.mark.asyncio
    async def test_not_found_message_byte_identical_for_uuid(self, mock_services):
        """36-char UUID → error message byte-identical to prior behavior."""
        mock_services["session_service"].get.return_value = None

        with pytest.raises(HandlerError) as exc:
            await handle_get_session(id=SID, session_service=mock_services["session_service"])

        assert str(exc.value) == f"Resource not found: {SID}"

    @pytest.mark.asyncio
    async def test_log_binding_byte_identical_for_uuid(self, mock_services, caplog):
        """36-char UUID → log id field byte-identical to prior behavior."""
        mock_services["session_service"].get.return_value = None

        with caplog.at_level("DEBUG"):
            with pytest.raises(HandlerError):
                await handle_get_session(
                    id=SID, session_service=mock_services["session_service"]
                )

        joined = " ".join(_plain_message(r) for r in caplog.records)
        assert f"session_id={SID}" in joined, "UUID altered in log binding"

    @pytest.mark.asyncio
    async def test_store_memory_not_found_message_unchanged_for_uuid(
        self, mock_services
    ):
        """store_memory not-found with valid UUID → frozen message unchanged."""
        mock_services["session_service"].get.return_value = None

        with pytest.raises(HandlerError) as exc:
            await handle_store_memory(
                session_id=SID,
                role="user",
                content="hello",
                memory_service=mock_services["memory_service"],
                session_service=mock_services["session_service"],
            )

        assert str(exc.value) == f"Resource not found: {SID}"


# ── Boundary sizes (EC-HIB-001..005) ─────────────────────────────────────


class TestBoundarySizes:
    @pytest.mark.asyncio
    async def test_not_found_64_char_id_unchanged(self, mock_services):
        """id exactly 64 chars → unchanged (EC-HIB-003)."""
        id64 = "a" * 64
        mock_services["session_service"].get.return_value = None

        with pytest.raises(HandlerError) as exc:
            await handle_get_session(
                id=id64, session_service=mock_services["session_service"]
            )

        assert str(exc.value) == f"Resource not found: {id64}"

    @pytest.mark.asyncio
    async def test_not_found_65_char_id_truncated_to_64(self, mock_services):
        """id 65 chars → echoed as 64-char bounded fragment (EC-HIB-004)."""
        id65 = "b" * 65
        mock_services["session_service"].get.return_value = None

        with pytest.raises(HandlerError) as exc:
            await handle_get_session(
                id=id65, session_service=mock_services["session_service"]
            )

        message = str(exc.value)
        assert id65 not in message
        echoed = message.removeprefix("Resource not found: ")
        assert len(echoed) == 64
        assert echoed == f"{'b' * 63}…"

    @pytest.mark.asyncio
    async def test_not_found_empty_id_no_crash(self, mock_services):
        """empty id → bounded path handles it (EC-HIB-002)."""
        mock_services["session_service"].get.return_value = None

        with pytest.raises(HandlerError) as exc:
            await handle_get_session(
                id="", session_service=mock_services["session_service"]
            )

        assert str(exc.value) == "Resource not found: "

    @pytest.mark.asyncio
    async def test_not_found_none_id_no_crash(self, mock_services):
        """id None → bounded path handles it (EC-HIB-001)."""
        mock_services["session_service"].get.return_value = None

        with pytest.raises(HandlerError) as exc:
            await handle_get_session(
                id=None, session_service=mock_services["session_service"]
            )

        assert str(exc.value) == "Resource not found: None"

    @pytest.mark.asyncio
    async def test_not_found_non_string_id_no_crash(self, mock_services):
        """non-string id → coerced and bounded without crash (EC-HIB-005)."""
        mock_services["session_service"].get.return_value = None

        with pytest.raises(HandlerError) as exc:
            await handle_get_session(
                id=12345, session_service=mock_services["session_service"]
            )

        assert str(exc.value) == "Resource not found: 12345"
