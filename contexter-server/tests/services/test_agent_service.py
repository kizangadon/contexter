"""Tests for AgentService.

The service is the translation boundary between the domain models and the
Rust engine's serde contract. Engine payloads are camelCase JSON with an
opaque ``config`` blob that carries the domain's LLM provider settings.
"""

from unittest.mock import AsyncMock

import pytest

from contexter_server.models.agent import AgentCreate, AgentPatch
from contexter_server.services.agent_service import AgentService


@pytest.fixture
def mock_engine():
    engine = AsyncMock()
    return engine


@pytest.fixture
def service(mock_engine):
    return AgentService(mock_engine)


@pytest.fixture
def engine_agent(any_uuid: str) -> dict:
    """A realistic engine Agent payload (what contexter_core returns)."""
    return {
        "id": any_uuid,
        "name": "Test Agent",
        "type": "chat",
        "description": "A test agent",
        "capabilities": ["search"],
        "status": "active",
        "config": {"provider": "openai", "model": "gpt-4", "temperature": 0.7},
        "version": 1,
        "createdAt": "2026-07-25T10:00:00Z",
        "updatedAt": "2026-07-25T10:00:00Z",
    }


class TestAgentServiceCreate:
    """Tests for AgentService.create."""

    @pytest.mark.asyncio
    async def test_creates_agent_with_engine_payload(self, service, mock_engine, engine_agent):
        """Create must send the engine's required fields and the config blob."""
        mock_engine.create_agent.return_value = engine_agent
        data = AgentCreate(
            name="Test Agent",
            provider="openai",
            model="gpt-4",
            type="chat",
            description="A test agent",
            capabilities=["search"],
        )
        result = await service.create(data)
        assert str(result.id) == engine_agent["id"]
        assert result.name == "Test Agent"
        assert result.provider == "openai"
        assert result.model == "gpt-4"
        assert result.type == "chat"
        assert result.status == "active"

        payload = mock_engine.create_agent.call_args[0][0]
        assert payload["name"] == "Test Agent"
        assert payload["type"] == "chat"  # engine requires `type`
        assert payload["description"] == "A test agent"  # engine requires `description`
        assert payload["capabilities"] == ["search"]
        assert payload["status"] == "active"
        assert payload["config"]["provider"] == "openai"
        assert payload["config"]["model"] == "gpt-4"

    @pytest.mark.asyncio
    async def test_create_defaults_engine_required_fields(self, service, mock_engine, any_uuid):
        """Create without type/description must default them for the engine."""
        bare_agent = {
            "id": any_uuid,
            "name": "Bare Agent",
            "type": "general",
            "description": "",
            "capabilities": [],
            "status": "active",
            "config": {},
            "version": 1,
            "createdAt": "2026-07-25T10:00:00Z",
            "updatedAt": "2026-07-25T10:00:00Z",
        }
        mock_engine.create_agent.return_value = bare_agent
        data = AgentCreate(name="Bare Agent")
        result = await service.create(data)

        payload = mock_engine.create_agent.call_args[0][0]
        assert payload["type"] == "general"
        assert payload["description"] == ""
        assert payload["status"] == "active"
        assert result.provider is None
        assert result.model is None

    @pytest.mark.asyncio
    async def test_create_serializes_system_prompt_camel_case_in_config(
        self, service, mock_engine, engine_agent
    ):
        """Nested config keys must already be camelCase (bridge is top-level only)."""
        mock_engine.create_agent.return_value = engine_agent
        data = AgentCreate(
            name="Prompted Agent",
            system_prompt="Be concise.",
            max_tokens=2048,
            metadata={"team": "core"},
        )
        await service.create(data)

        payload = mock_engine.create_agent.call_args[0][0]
        config = payload["config"]
        assert config["systemPrompt"] == "Be concise."
        assert config["maxTokens"] == 2048
        assert config["metadata"] == {"team": "core"}


class TestAgentServiceGet:
    """Tests for AgentService.get."""

    @pytest.mark.asyncio
    async def test_gets_agent_resolves_config(self, service, mock_engine, engine_agent):
        """Get must resolve provider/model from the engine config blob."""
        mock_engine.get_agent.return_value = engine_agent
        result = await service.get(engine_agent["id"])
        assert result is not None
        assert result.name == "Test Agent"
        assert result.provider == "openai"
        assert result.model == "gpt-4"
        assert result.type == "chat"
        assert result.status == "active"
        assert result.version == 1
        assert result.capabilities == ["search"]

    @pytest.mark.asyncio
    async def test_gets_agent_legacy_flat_payload(self, service, mock_engine, any_uuid):
        """Legacy flat payloads (provider/model at top level) must still parse."""
        mock_engine.get_agent.return_value = {
            "id": any_uuid,
            "name": "Flat Agent",
            "provider": "anthropic",
            "model": "claude-3",
            "tools": ["code"],
            "created_at": "2026-07-25T10:00:00Z",
            "updated_at": "2026-07-25T10:00:00Z",
        }
        result = await service.get(any_uuid)
        assert result is not None
        assert result.provider == "anthropic"
        assert result.model == "claude-3"
        assert result.capabilities == ["code"]

    @pytest.mark.asyncio
    async def test_get_returns_none_when_not_found(self, service, mock_engine):
        mock_engine.get_agent.return_value = None
        result = await service.get("nonexistent")
        assert result is None


class TestAgentServiceList:
    """Tests for AgentService.list."""

    @pytest.mark.asyncio
    async def test_lists_agents(self, service, mock_engine, engine_agent, any_uuid):
        aid2 = any_uuid.replace("000001", "000003")
        second = dict(engine_agent, id=aid2, name="Agent 2")
        mock_engine.list_agents.return_value = [engine_agent, second]
        result = await service.list()
        assert len(result) == 2
        assert result[0].name == "Test Agent"
        assert result[1].provider == "openai"

    @pytest.mark.asyncio
    async def test_list_returns_empty(self, service, mock_engine):
        mock_engine.list_agents.return_value = []
        result = await service.list()
        assert result == []


class TestAgentServiceUpdate:
    """Tests for AgentService.update."""

    @pytest.mark.asyncio
    async def test_updates_agent(self, service, mock_engine, engine_agent):
        """A name-only patch must translate to a name-only engine patch."""
        mock_engine.get_agent.return_value = engine_agent
        updated = dict(engine_agent, name="Updated Agent")
        mock_engine.update_agent.return_value = updated
        patch = AgentPatch(name="Updated Agent")
        result = await service.update(engine_agent["id"], patch)
        assert result is not None
        assert result.name == "Updated Agent"
        mock_engine.update_agent.assert_awaited_once_with(
            engine_agent["id"], {"name": "Updated Agent"}
        )

    @pytest.mark.asyncio
    async def test_update_merges_config_preserving_untouched_fields(
        self, service, mock_engine, engine_agent
    ):
        """A model-only patch must keep the existing config's provider."""
        mock_engine.get_agent.return_value = engine_agent
        updated = dict(engine_agent)
        mock_engine.update_agent.return_value = updated
        patch = AgentPatch(model="gpt-4o")
        await service.update(engine_agent["id"], patch)

        payload = mock_engine.update_agent.call_args[0][1]
        assert payload["config"]["provider"] == "openai"  # preserved
        assert payload["config"]["model"] == "gpt-4o"  # replaced

    @pytest.mark.asyncio
    async def test_update_translates_type(self, service, mock_engine, engine_agent):
        """Engine-aligned fields pass through the engine patch verbatim."""
        mock_engine.get_agent.return_value = engine_agent
        mock_engine.update_agent.return_value = dict(engine_agent)
        patch = AgentPatch(type="coding-assistant", status="inactive")
        await service.update(engine_agent["id"], patch)

        payload = mock_engine.update_agent.call_args[0][1]
        assert payload["type"] == "coding-assistant"
        assert payload["status"] == "inactive"

    @pytest.mark.asyncio
    async def test_update_returns_none_when_missing(self, service, mock_engine):
        """A config-bearing patch on a missing agent must return None without
        calling the engine's update (the config cannot be merged)."""
        mock_engine.get_agent.return_value = None
        result = await service.update("nonexistent", AgentPatch(model="gpt-4o"))
        assert result is None
        mock_engine.update_agent.assert_not_awaited()

    @pytest.mark.asyncio
    async def test_update_returns_none_when_empty(self, service, mock_engine, engine_agent):
        mock_engine.get_agent.return_value = engine_agent
        mock_engine.update_agent.return_value = {}
        patch = AgentPatch(name="Updated")
        result = await service.update(engine_agent["id"], patch)
        assert result is None


class TestAgentServiceDelete:
    """Tests for AgentService.delete."""

    @pytest.mark.asyncio
    async def test_deletes_agent(self, service, mock_engine, any_uuid):
        await service.delete(any_uuid)
        mock_engine.delete_agent.assert_awaited_once_with(any_uuid)
