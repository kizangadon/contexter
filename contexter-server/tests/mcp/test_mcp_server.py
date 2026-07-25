"""Tests for the Contexter MCP server tools and resources.

Tests the pure handler functions in ``mcp_tools.handlers`` directly with
mocked service instances.
"""

import os
from unittest import mock
from unittest.mock import AsyncMock
from uuid import UUID

import pytest

from contexter_server.mcp_server import create_mcp_server
from contexter_server.mcp_tools.auth import MCPAuthError
from contexter_server.mcp_tools.handlers import (
    handle_agent_resource,
    handle_export_data,
    handle_get_agent_info,
    handle_get_session,
    handle_get_system_health,
    handle_list_recent_sessions,
    handle_list_skills,
    handle_memory_resource,
    handle_search_memories,
    handle_session_resource,
    handle_analytics_overview_resource,
    handle_store_memory,
)
from contexter_server.models.agent import Agent
from contexter_server.models.analytics import SystemHealth, AnalyticsOverview
from contexter_server.models.export import ExportStatus
from contexter_server.models.memory import Memory
from contexter_server.models.search import SearchResponse, SearchResult
from contexter_server.models.session import Session


# ── Fixtures ────────────────────────────────────────────────────────────


@pytest.fixture
def mock_services():
    """Create a dict of fully-mocked service objects."""
    return {
        "memory_service": AsyncMock(),
        "session_service": AsyncMock(),
        "agent_service": AsyncMock(),
        "skill_service": AsyncMock(),
        "analytics_service": AsyncMock(),
        "export_service": AsyncMock(),
    }


@pytest.fixture
def sample_session(any_uuid):
    return Session(
        id=UUID(any_uuid),
        agent_id=UUID(any_uuid),
        project="test-project",
        name="Test Session",
    )


@pytest.fixture
def sample_memory(any_uuid):
    return Memory(
        id=UUID(any_uuid),
        session_id=UUID(any_uuid),
        agent_id=UUID(any_uuid),
        role="user",
        content="Hello, world!",
    )


@pytest.fixture
def sample_agent(any_uuid):
    return Agent(
        id=UUID(any_uuid),
        name="test-agent",
        provider="openai",
        model="gpt-4",
    )


# ── Tool Tests ──────────────────────────────────────────────────────────


class TestStoreMemory:
    """Tests for the store_memory tool handler."""

    @pytest.mark.asyncio
    async def test_stores_memory_successfully(self, mock_services, sample_session, sample_memory, any_uuid):
        """Should look up session, create memory, return memory_id and created_at."""
        mock_services["session_service"].get.return_value = sample_session
        mock_services["memory_service"].create.return_value = sample_memory

        result = await handle_store_memory(
            session_id=any_uuid,
            role="user",
            content="Hello, world!",
            memory_service=mock_services["memory_service"],
            session_service=mock_services["session_service"],
        )

        assert "error" not in result
        assert result["memory_id"] == any_uuid
        assert "created_at" in result
        mock_services["session_service"].get.assert_awaited_once_with(any_uuid)
        mock_services["memory_service"].create.assert_awaited_once()

    @pytest.mark.asyncio
    async def test_returns_error_when_session_not_found(self, mock_services, any_uuid):
        """Should return error when session does not exist."""
        mock_services["session_service"].get.return_value = None

        result = await handle_store_memory(
            session_id=any_uuid,
            role="user",
            content="Hello",
            memory_service=mock_services["memory_service"],
            session_service=mock_services["session_service"],
        )

        assert "error" in result
        assert "not found" in result["error"]

    @pytest.mark.asyncio
    async def test_returns_error_for_invalid_uuid(self, mock_services, any_uuid):
        """Should return error when session_id is not a valid UUID."""
        mock_services["session_service"].get.return_value = None

        result = await handle_store_memory(
            session_id="not-a-uuid",
            role="user",
            content="Hello",
            memory_service=mock_services["memory_service"],
            session_service=mock_services["session_service"],
        )

        assert "error" in result
        assert "uuid" in result["error"].lower() or "invalid" in result["error"].lower()

    @pytest.mark.asyncio
    async def test_returns_error_when_services_unavailable(self, any_uuid):
        """Should return error when services are None."""
        result = await handle_store_memory(
            session_id=any_uuid,
            role="user",
            content="Hello",
        )

        assert "error" in result


class TestSearchMemories:
    """Tests for the search_memories tool handler."""

    @pytest.mark.asyncio
    async def test_returns_search_results(self, mock_services, any_uuid):
        """Should return formatted search results with total count."""
        mock_services["memory_service"].search.return_value = SearchResponse(
            results=[
                SearchResult(
                    id=UUID(any_uuid),
                    type="memory",
                    score=0.95,
                    snippet="test content",
                ),
            ],
            total=1,
        )

        result = await handle_search_memories(
            query="test",
            memory_service=mock_services["memory_service"],
        )

        assert "error" not in result
        assert len(result["results"]) == 1
        assert result["total"] == 1
        assert result["results"][0]["snippet"] == "test content"

    @pytest.mark.asyncio
    async def test_passes_optional_filters(self, mock_services):
        """Should pass type, project, limit to the search query."""
        mock_services["memory_service"].search.return_value = SearchResponse()

        await handle_search_memories(
            query="test",
            type_filter="user",
            project="my-project",
            limit=10,
            memory_service=mock_services["memory_service"],
        )

        call_kwargs = mock_services["memory_service"].search.call_args[0][0]
        assert call_kwargs.query == "test"
        assert call_kwargs.type == "user"
        assert call_kwargs.project == "my-project"
        assert call_kwargs.limit == 10

    @pytest.mark.asyncio
    async def test_returns_error_when_service_unavailable(self):
        """Should return error when memory_service is None."""
        result = await handle_search_memories(query="test")
        assert "error" in result


class TestGetSession:
    """Tests for the get_session tool handler."""

    @pytest.mark.asyncio
    async def test_returns_session(self, mock_services, sample_session, any_uuid):
        """Should return session data."""
        mock_services["session_service"].get.return_value = sample_session

        result = await handle_get_session(
            id=any_uuid,
            session_service=mock_services["session_service"],
        )

        assert "error" not in result
        assert result["session"]["name"] == "Test Session"
        assert result["session"]["project"] == "test-project"

    @pytest.mark.asyncio
    async def test_returns_error_when_not_found(self, mock_services):
        """Should return error for missing session."""
        mock_services["session_service"].get.return_value = None

        result = await handle_get_session(
            id="nonexistent",
            session_service=mock_services["session_service"],
        )

        assert "error" in result
        assert "not found" in result["error"]

    @pytest.mark.asyncio
    async def test_returns_error_when_service_unavailable(self, any_uuid):
        result = await handle_get_session(id=any_uuid)
        assert "error" in result


class TestListRecentSessions:
    """Tests for the list_recent_sessions tool handler."""

    @pytest.mark.asyncio
    async def test_returns_all_sessions(self, mock_services, sample_session, any_uuid):
        """Should return all sessions when no limit is given."""
        mock_services["session_service"].list.return_value = [sample_session]

        result = await handle_list_recent_sessions(
            session_service=mock_services["session_service"],
        )

        assert "error" not in result
        assert len(result["sessions"]) == 1
        assert result["sessions"][0]["name"] == "Test Session"

    @pytest.mark.asyncio
    async def test_respects_limit(self, mock_services, sample_session, any_uuid):
        """Should limit returned sessions."""
        alt_id = any_uuid.replace("-1", "-2") if "-1" in any_uuid else any_uuid
        session2 = Session(
            id=UUID(alt_id),
            agent_id=UUID(any_uuid),
            project="test-project",
        )
        mock_services["session_service"].list.return_value = [sample_session, session2]

        result = await handle_list_recent_sessions(
            limit=1,
            session_service=mock_services["session_service"],
        )

        assert len(result["sessions"]) == 1

    @pytest.mark.asyncio
    async def test_filters_by_project(self, mock_services):
        """Should pass project filter to service."""
        mock_services["session_service"].list.return_value = []

        await handle_list_recent_sessions(
            project="my-project",
            session_service=mock_services["session_service"],
        )

        call_filter = mock_services["session_service"].list.call_args[1]["filter"]
        assert call_filter is not None
        assert call_filter.project == "my-project"

    @pytest.mark.asyncio
    async def test_returns_error_when_service_unavailable(self):
        result = await handle_list_recent_sessions()
        assert "error" in result


class TestGetAgentInfo:
    """Tests for the get_agent_info tool handler."""

    @pytest.mark.asyncio
    async def test_returns_agent(self, mock_services, sample_agent, any_uuid):
        """Should return agent data."""
        mock_services["agent_service"].get.return_value = sample_agent

        result = await handle_get_agent_info(
            id=any_uuid,
            agent_service=mock_services["agent_service"],
        )

        assert "error" not in result
        assert result["agent"]["name"] == "test-agent"

    @pytest.mark.asyncio
    async def test_returns_error_when_not_found(self, mock_services):
        mock_services["agent_service"].get.return_value = None

        result = await handle_get_agent_info(
            id="nonexistent",
            agent_service=mock_services["agent_service"],
        )

        assert "error" in result
        assert "not found" in result["error"]

    @pytest.mark.asyncio
    async def test_returns_error_when_service_unavailable(self, any_uuid):
        result = await handle_get_agent_info(id=any_uuid)
        assert "error" in result


class TestListSkills:
    """Tests for the list_skills tool handler."""

    @pytest.mark.asyncio
    async def test_returns_all_skills(self, mock_services, any_uuid):
        """Should return all skills."""
        from contexter_server.models.skill import Skill
        skill = Skill(
            id=UUID(any_uuid),
            name="test-skill",
            type="memory",
        )
        mock_services["skill_service"].list.return_value = [skill]

        result = await handle_list_skills(
            skill_service=mock_services["skill_service"],
        )

        assert "error" not in result
        assert len(result["skills"]) == 1
        assert result["skills"][0]["name"] == "test-skill"

    @pytest.mark.asyncio
    async def test_filters_by_type(self, mock_services):
        """Should pass type filter to service."""
        mock_services["skill_service"].list.return_value = []

        await handle_list_skills(
            type_filter="search",
            skill_service=mock_services["skill_service"],
        )

        call_filter = mock_services["skill_service"].list.call_args[1]["filter"]
        assert call_filter == {"type": "search"}

    @pytest.mark.asyncio
    async def test_returns_error_when_service_unavailable(self):
        result = await handle_list_skills()
        assert "error" in result


class TestGetSystemHealth:
    """Tests for the get_system_health tool handler."""

    @pytest.mark.asyncio
    async def test_returns_health_data(self, mock_services):
        """Should return formatted system health."""
        mock_services["analytics_service"].get_health.return_value = SystemHealth(
            status="ok",
            uptime_seconds=3600,
            memory_usage_mb=128.5,
            storage_size_bytes=1024000,
        )

        result = await handle_get_system_health(
            analytics_service=mock_services["analytics_service"],
        )

        assert "error" not in result
        assert result["status"] == "ok"
        assert result["uptime"] == 3600
        assert result["memory_usage"] == 128.5
        assert result["storage_size"] == 1024000

    @pytest.mark.asyncio
    async def test_returns_error_when_service_unavailable(self):
        result = await handle_get_system_health()
        assert "error" in result


class TestExportData:
    """Tests for the export_data tool handler."""

    @pytest.mark.asyncio
    async def test_submits_export(self, mock_services):
        """Should submit an export request and return status."""
        mock_services["export_service"].submit.return_value = ExportStatus(
            id=UUID("00000000-0000-0000-0000-000000000001"),
            status="completed",
        )

        result = await handle_export_data(
            format="json",
            entities=["sessions", "memories"],
            export_service=mock_services["export_service"],
        )

        assert "error" not in result
        assert result["export_id"] == "00000000-0000-0000-0000-000000000001"
        assert result["status"] == "completed"

    @pytest.mark.asyncio
    async def test_returns_error_when_service_unavailable(self):
        result = await handle_export_data()
        assert "error" in result


# ── Resource Tests ──────────────────────────────────────────────────────


class TestSessionResource:
    """Tests for the contexter://session/{id} resource."""

    @pytest.mark.asyncio
    async def test_returns_session_json(self, mock_services, sample_session, any_uuid):
        """Should return formatted session JSON."""
        mock_services["session_service"].get.return_value = sample_session

        result = await handle_session_resource(
            id=any_uuid,
            session_service=mock_services["session_service"],
        )

        assert result.startswith("{")
        assert "Test Session" in result

    @pytest.mark.asyncio
    async def test_returns_not_found_message(self, mock_services):
        mock_services["session_service"].get.return_value = None

        result = await handle_session_resource(
            id="nonexistent",
            session_service=mock_services["session_service"],
        )

        assert result == "Session not found"

    @pytest.mark.asyncio
    async def test_returns_error_when_service_unavailable(self, any_uuid):
        result = await handle_session_resource(id=any_uuid)
        assert "not connected" in result


class TestMemoryResource:
    """Tests for the contexter://memory/{id} resource."""

    @pytest.mark.asyncio
    async def test_returns_memory_json(self, mock_services, sample_memory, any_uuid):
        mock_services["memory_service"].get.return_value = sample_memory

        result = await handle_memory_resource(
            id=any_uuid,
            memory_service=mock_services["memory_service"],
        )

        assert result.startswith("{")
        assert "Hello, world!" in result

    @pytest.mark.asyncio
    async def test_returns_not_found_message(self, mock_services):
        mock_services["memory_service"].get.return_value = None

        result = await handle_memory_resource(
            id="nonexistent",
            memory_service=mock_services["memory_service"],
        )

        assert result == "Memory not found"

    @pytest.mark.asyncio
    async def test_returns_error_when_service_unavailable(self, any_uuid):
        result = await handle_memory_resource(id=any_uuid)
        assert "not connected" in result


class TestAgentResource:
    """Tests for the contexter://agent/{id} resource."""

    @pytest.mark.asyncio
    async def test_returns_agent_json(self, mock_services, sample_agent, any_uuid):
        mock_services["agent_service"].get.return_value = sample_agent

        result = await handle_agent_resource(
            id=any_uuid,
            agent_service=mock_services["agent_service"],
        )

        assert result.startswith("{")
        assert "test-agent" in result

    @pytest.mark.asyncio
    async def test_returns_not_found_message(self, mock_services):
        mock_services["agent_service"].get.return_value = None

        result = await handle_agent_resource(
            id="nonexistent",
            agent_service=mock_services["agent_service"],
        )

        assert result == "Agent not found"

    @pytest.mark.asyncio
    async def test_returns_error_when_service_unavailable(self, any_uuid):
        result = await handle_agent_resource(id=any_uuid)
        assert "not connected" in result


class TestAnalyticsOverviewResource:
    """Tests for the contexter://analytics/overview resource."""

    @pytest.mark.asyncio
    async def test_returns_overview_json(self, mock_services):
        mock_services["analytics_service"].get_overview.return_value = AnalyticsOverview(
            total_sessions=10,
            total_memories=100,
            total_agents=3,
            total_skills=5,
            storage_size_bytes=2048000,
            uptime_seconds=7200,
        )

        result = await handle_analytics_overview_resource(
            analytics_service=mock_services["analytics_service"],
        )

        assert result.startswith("{")
        assert '"total_sessions": 10' in result

    @pytest.mark.asyncio
    async def test_returns_error_when_service_unavailable(self):
        result = await handle_analytics_overview_resource()
        assert "not connected" in result


# ── Server Creation Tests ──────────────────────────────────────────────


# ── Tool Auth Tests ─────────────────────────────────────────────────────


class TestToolAuth:
    """Tests for tool handler authentication enforcement."""

    @pytest.fixture(autouse=True)
    def _patch_env(self):
        """Set CONtexTER_API_KEY for auth tests (restore after)."""
        with mock.patch.dict(os.environ, {"CONtexTER_API_KEY": "test-key-123"}):
            yield

    @pytest.mark.asyncio
    async def test_store_memory_rejects_missing_key(self, mock_services, any_uuid):
        """store_memory handler rejects call when _api_key is missing."""
        with pytest.raises(MCPAuthError, match="API key required"):
            await handle_store_memory(
                session_id=any_uuid,
                role="user",
                content="test",
                memory_service=mock_services["memory_service"],
                session_service=mock_services["session_service"],
            )

    @pytest.mark.asyncio
    async def test_store_memory_rejects_wrong_key(self, mock_services, any_uuid):
        """store_memory handler rejects call with wrong _api_key."""
        with pytest.raises(MCPAuthError, match="Invalid API key"):
            await handle_store_memory(
                session_id=any_uuid,
                role="user",
                content="test",
                _api_key="wrong-key",
                memory_service=mock_services["memory_service"],
                session_service=mock_services["session_service"],
            )

    @pytest.mark.asyncio
    async def test_store_memory_accepts_valid_key(self, mock_services, sample_session, sample_memory, any_uuid):
        """store_memory handler accepts call with correct _api_key."""
        mock_services["session_service"].get.return_value = sample_session
        mock_services["memory_service"].create.return_value = sample_memory

        result = await handle_store_memory(
            session_id=any_uuid,
            role="user",
            content="test",
            _api_key="test-key-123",
            memory_service=mock_services["memory_service"],
            session_service=mock_services["session_service"],
        )

        assert "error" not in result
        assert result["memory_id"] == any_uuid

    @pytest.mark.asyncio
    async def test_search_memories_rejects_missing_key(self, mock_services):
        """search_memories handler rejects call when _api_key is missing."""
        with pytest.raises(MCPAuthError, match="API key required"):
            await handle_search_memories(
                query="test",
                memory_service=mock_services["memory_service"],
            )

    @pytest.mark.asyncio
    async def test_get_session_rejects_missing_key(self, mock_services, any_uuid):
        """get_session handler rejects call when _api_key is missing."""
        with pytest.raises(MCPAuthError, match="API key required"):
            await handle_get_session(
                id=any_uuid,
                session_service=mock_services["session_service"],
            )

    @pytest.mark.asyncio
    async def test_list_recent_sessions_rejects_missing_key(self, mock_services):
        """list_recent_sessions handler rejects call when _api_key is missing."""
        with pytest.raises(MCPAuthError, match="API key required"):
            await handle_list_recent_sessions(
                session_service=mock_services["session_service"],
            )

    @pytest.mark.asyncio
    async def test_get_agent_info_rejects_missing_key(self, mock_services, any_uuid):
        """get_agent_info handler rejects call when _api_key is missing."""
        with pytest.raises(MCPAuthError, match="API key required"):
            await handle_get_agent_info(
                id=any_uuid,
                agent_service=mock_services["agent_service"],
            )

    @pytest.mark.asyncio
    async def test_list_skills_rejects_missing_key(self, mock_services):
        """list_skills handler rejects call when _api_key is missing."""
        with pytest.raises(MCPAuthError, match="API key required"):
            await handle_list_skills(
                skill_service=mock_services["skill_service"],
            )

    @pytest.mark.asyncio
    async def test_get_system_health_rejects_missing_key(self, mock_services):
        """get_system_health handler rejects call when _api_key is missing."""
        with pytest.raises(MCPAuthError, match="API key required"):
            await handle_get_system_health(
                analytics_service=mock_services["analytics_service"],
            )

    @pytest.mark.asyncio
    async def test_export_data_rejects_missing_key(self, mock_services):
        """export_data handler rejects call when _api_key is missing."""
        with pytest.raises(MCPAuthError, match="API key required"):
            await handle_export_data(
                export_service=mock_services["export_service"],
            )


class TestResourceAuth:
    """Tests for resource handler authentication enforcement."""

    @pytest.fixture(autouse=True)
    def _patch_env(self):
        """Set CONtexTER_API_KEY for auth tests (restore after)."""
        with mock.patch.dict(os.environ, {"CONtexTER_API_KEY": "test-key-123"}):
            yield

    @pytest.mark.asyncio
    async def test_session_resource_rejects_missing_key(self, mock_services, any_uuid):
        """session_resource handler rejects call when _api_key is missing."""
        with pytest.raises(MCPAuthError, match="API key required"):
            await handle_session_resource(
                id=any_uuid,
                session_service=mock_services["session_service"],
            )

    @pytest.mark.asyncio
    async def test_session_resource_rejects_wrong_key(self, mock_services, any_uuid):
        """session_resource handler rejects call with wrong _api_key."""
        with pytest.raises(MCPAuthError, match="Invalid API key"):
            await handle_session_resource(
                id=any_uuid,
                _api_key="wrong-key",
                session_service=mock_services["session_service"],
            )

    @pytest.mark.asyncio
    async def test_session_resource_accepts_valid_key(self, mock_services, sample_session, any_uuid):
        """session_resource handler accepts call with correct _api_key."""
        mock_services["session_service"].get.return_value = sample_session

        result = await handle_session_resource(
            id=any_uuid,
            _api_key="test-key-123",
            session_service=mock_services["session_service"],
        )

        assert result.startswith("{")
        assert "Test Session" in result

    @pytest.mark.asyncio
    async def test_memory_resource_rejects_missing_key(self, mock_services, any_uuid):
        """memory_resource handler rejects call when _api_key is missing."""
        with pytest.raises(MCPAuthError, match="API key required"):
            await handle_memory_resource(
                id=any_uuid,
                memory_service=mock_services["memory_service"],
            )

    @pytest.mark.asyncio
    async def test_memory_resource_rejects_wrong_key(self, mock_services, any_uuid):
        """memory_resource handler rejects call with wrong _api_key."""
        with pytest.raises(MCPAuthError, match="Invalid API key"):
            await handle_memory_resource(
                id=any_uuid,
                _api_key="wrong-key",
                memory_service=mock_services["memory_service"],
            )

    @pytest.mark.asyncio
    async def test_memory_resource_accepts_valid_key(self, mock_services, sample_memory, any_uuid):
        """memory_resource handler accepts call with correct _api_key."""
        mock_services["memory_service"].get.return_value = sample_memory

        result = await handle_memory_resource(
            id=any_uuid,
            _api_key="test-key-123",
            memory_service=mock_services["memory_service"],
        )

        assert result.startswith("{")
        assert "Hello, world!" in result

    @pytest.mark.asyncio
    async def test_agent_resource_rejects_missing_key(self, mock_services, any_uuid):
        """agent_resource handler rejects call when _api_key is missing."""
        with pytest.raises(MCPAuthError, match="API key required"):
            await handle_agent_resource(
                id=any_uuid,
                agent_service=mock_services["agent_service"],
            )

    @pytest.mark.asyncio
    async def test_agent_resource_rejects_wrong_key(self, mock_services, any_uuid):
        """agent_resource handler rejects call with wrong _api_key."""
        with pytest.raises(MCPAuthError, match="Invalid API key"):
            await handle_agent_resource(
                id=any_uuid,
                _api_key="wrong-key",
                agent_service=mock_services["agent_service"],
            )

    @pytest.mark.asyncio
    async def test_agent_resource_accepts_valid_key(self, mock_services, sample_agent, any_uuid):
        """agent_resource handler accepts call with correct _api_key."""
        mock_services["agent_service"].get.return_value = sample_agent

        result = await handle_agent_resource(
            id=any_uuid,
            _api_key="test-key-123",
            agent_service=mock_services["agent_service"],
        )

        assert result.startswith("{")
        assert "test-agent" in result

    @pytest.mark.asyncio
    async def test_analytics_overview_resource_rejects_missing_key(self, mock_services):
        """analytics_overview_resource handler rejects call when _api_key is missing."""
        with pytest.raises(MCPAuthError, match="API key required"):
            await handle_analytics_overview_resource(
                analytics_service=mock_services["analytics_service"],
            )

    @pytest.mark.asyncio
    async def test_analytics_overview_resource_rejects_wrong_key(self, mock_services):
        """analytics_overview_resource handler rejects call with wrong _api_key."""
        with pytest.raises(MCPAuthError, match="Invalid API key"):
            await handle_analytics_overview_resource(
                _api_key="wrong-key",
                analytics_service=mock_services["analytics_service"],
            )

    @pytest.mark.asyncio
    async def test_analytics_overview_resource_accepts_valid_key(self, mock_services):
        """analytics_overview_resource handler accepts call with correct _api_key."""
        mock_services["analytics_service"].get_overview.return_value = AnalyticsOverview(
            total_sessions=10,
            total_memories=100,
            total_agents=3,
            total_skills=5,
            storage_size_bytes=2048000,
            uptime_seconds=7200,
        )

        result = await handle_analytics_overview_resource(
            _api_key="test-key-123",
            analytics_service=mock_services["analytics_service"],
        )

        assert result.startswith("{")
        assert '"total_sessions": 10' in result


class TestCreateMCPServer:
    """Tests for the FastMCP server factory."""

    def test_creates_server_without_services(self):
        """Should create a server even without service instances."""
        mcp = create_mcp_server()
        assert mcp is not None

    def test_creates_server_with_services(self, mock_services):
        """Should create a server with all service instances."""
        mcp = create_mcp_server(**mock_services)
        assert mcp is not None
        assert mcp.name == "contexter"


class TestMCPServerLifecycle:
    """Tests for MCP server lifecycle in the FastAPI app."""

    def test_app_stores_mcp_thread_and_event(self):
        """create_app should store mcp_thread and mcp_shutdown_event on app.state."""
        from contexter_server.main import create_app
        app = create_app(data_path="/tmp/contexter-test-lifecycle")

        # App is created but lifespan hasn't run yet (no thread/event)
        assert not hasattr(app.state, "mcp_thread") or app.state.mcp_thread is None

    def test_lifespan_sets_mcp_state(self):
        """Entering the lifespan context should start the MCP thread."""
        from contexter_server.main import create_app
        from fastapi.testclient import TestClient

        app = create_app(data_path="/tmp/contexter-test-lifecycle")

        with TestClient(app) as client:
            thread = client.app.state.mcp_thread
            event = client.app.state.mcp_shutdown_event
            assert thread is not None
            assert thread.is_alive()
            assert event is not None
            assert not event.is_set()

        # After lifespan exit, the shutdown event should be set
        assert event.is_set()

    def test_lifespan_shutdown_joins_thread(self):
        """Exiting the lifespan should join the MCP thread with a timeout."""
        from contexter_server.main import create_app
        from fastapi.testclient import TestClient

        app = create_app(data_path="/tmp/contexter-test-lifecycle")

        with TestClient(app) as client:
            thread = client.app.state.mcp_thread

        # After lifespan exit, thread should have been joined (daemon + join)
        # Since run() blocks, the daemon thread exits when the interpreter
        # shuts down; the join() call in the lifespan waits up to 5 s.
        assert not thread.is_alive()
