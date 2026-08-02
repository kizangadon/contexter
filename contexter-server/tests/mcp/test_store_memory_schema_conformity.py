"""RED reproduction tests — Bug 2026-08-01-store-memory-schema-conformity.

The frozen contract table declares store_memory parameters as exactly
``session_id``, ``role``, ``content``, ``_api_key``. The unfixed code
registers three extra optional parameters (``tokens``, ``tokenizer``,
``model``), creating schema drift.

These tests fail on the unfixed code and pass after the schema/signature
trim (REQ-SM-001..003, AC-SM-001..003, EC-SM-001..003).
"""

import inspect
from unittest.mock import AsyncMock
from uuid import UUID

import pytest

from contexter_server.mcp_server import create_mcp_server
from contexter_server.mcp_tools.handlers import handle_store_memory
from contexter_server.models.memory import Memory
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

EXPECTED_PARAMS = {"session_id", "role", "content", "_api_key"}
LEGACY_EXTRA_PARAMS = {"tokens", "tokenizer", "model"}


class TestStoreMemorySchema:
    @pytest.mark.asyncio
    async def test_registered_schema_declares_exact_frozen_params(self, mock_services):
        """AC-SM-001: tools/list → store_memory schema has exactly 4 params."""
        mcp = create_mcp_server(**mock_services)
        assert mcp is not None

        tools = await mcp.list_tools()
        by_name = {tool.name: tool for tool in tools}
        tool = by_name["store_memory"]
        schema = (
            tool.input_schema if hasattr(tool, "input_schema") else tool.parameters
        )
        properties = set(schema.get("properties", {}).keys())

        assert properties == EXPECTED_PARAMS, (
            f"store_memory schema must be exactly {sorted(EXPECTED_PARAMS)}, "
            f"got {sorted(properties)}"
        )

    @pytest.mark.asyncio
    async def test_registered_schema_has_no_legacy_extra_params(self, mock_services):
        """REQ-SM-001: tokens/tokenizer/model must not be advertised."""
        mcp = create_mcp_server(**mock_services)

        tools = await mcp.list_tools()
        by_name = {tool.name: tool for tool in tools}
        schema = (
            by_name["store_memory"].input_schema
            if hasattr(by_name["store_memory"], "input_schema")
            else by_name["store_memory"].parameters
        )
        properties = set(schema.get("properties", {}).keys())
        assert LEGACY_EXTRA_PARAMS.isdisjoint(properties), (
            f"legacy params {sorted(LEGACY_EXTRA_PARAMS & properties)} still advertised"
        )

    def test_handler_signature_matches_frozen_params(self):
        """REQ-SM-002: handler's client-facing params align with the schema.

        The handler additionally declares keyword-only service plumbing
        (``memory_service``, ``session_service``) used for direct-call tests
        and server wiring; those are not client-facing and never reach the
        registered schema (which AC-SM-001 asserts to be exactly the frozen
        set). The drift this contract targets is the legacy extras
        (``tokens``/``tokenizer``/``model``), which must not be present.
        """
        signature = inspect.signature(handle_store_memory)
        handler_params = set(signature.parameters.keys())
        assert EXPECTED_PARAMS.issubset(handler_params), (
            f"handler missing frozen params: {sorted(EXPECTED_PARAMS - handler_params)}"
        )
        assert LEGACY_EXTRA_PARAMS.isdisjoint(handler_params), (
            f"legacy params {sorted(LEGACY_EXTRA_PARAMS & handler_params)} "
            "still present in handler"
        )

    @pytest.mark.asyncio
    async def test_store_memory_still_works_with_frozen_params(self, mock_services):
        """EC-SM-001: valid call with only the frozen params succeeds."""
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
            content="Hello, world!",
        )
        mock_services["session_service"].get.return_value = session
        mock_services["memory_service"].create.return_value = memory

        result = await handle_store_memory(
            session_id=SID,
            role="user",
            content="Hello, world!",
            memory_service=mock_services["memory_service"],
            session_service=mock_services["session_service"],
        )
        assert "error" not in result
        assert result["memory_id"] == SID

        created = mock_services["memory_service"].create.call_args[0][0]
        assert created.content == "Hello, world!"
