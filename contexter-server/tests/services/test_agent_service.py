"""Tests for AgentService."""

from unittest.mock import AsyncMock

import pytest

from contexter_server.models.agent import Agent, AgentCreate, AgentPatch
from contexter_server.services.agent_service import AgentService


@pytest.fixture
def mock_engine():
    engine = AsyncMock()
    return engine


@pytest.fixture
def service(mock_engine):
    return AgentService(mock_engine)


class TestAgentServiceCreate:
    """Tests for AgentService.create."""

    @pytest.mark.asyncio
    async def test_creates_agent(self, service, mock_engine, any_uuid):
        mock_engine.create_agent.return_value = {
            "id": any_uuid,
            "name": "Test Agent",
            "provider": "openai",
            "model": "gpt-4",
        }
        data = AgentCreate(name="Test Agent", provider="openai", model="gpt-4")
        result = await service.create(data)
        assert str(result.id) == any_uuid
        assert result.name == "Test Agent"
        mock_engine.create_agent.assert_awaited_once()


class TestAgentServiceGet:
    """Tests for AgentService.get."""

    @pytest.mark.asyncio
    async def test_gets_agent(self, service, mock_engine, any_uuid):
        mock_engine.get_agent.return_value = {
            "id": any_uuid,
            "name": "Test Agent",
            "provider": "openai",
            "model": "gpt-4",
        }
        result = await service.get(any_uuid)
        assert result is not None
        assert result.name == "Test Agent"

    @pytest.mark.asyncio
    async def test_get_returns_none_when_not_found(self, service, mock_engine):
        mock_engine.get_agent.return_value = None
        result = await service.get("nonexistent")
        assert result is None


class TestAgentServiceList:
    """Tests for AgentService.list."""

    @pytest.mark.asyncio
    async def test_lists_agents(self, service, mock_engine, any_uuid):
        aid1 = any_uuid.replace("000001", "000002")
        aid2 = any_uuid.replace("000001", "000003")
        mock_engine.list_agents.return_value = [
            {"id": aid1, "name": "Agent 1", "provider": "openai", "model": "gpt-4"},
            {"id": aid2, "name": "Agent 2", "provider": "anthropic", "model": "claude-3"},
        ]
        result = await service.list()
        assert len(result) == 2
        assert result[0].name == "Agent 1"

    @pytest.mark.asyncio
    async def test_list_returns_empty(self, service, mock_engine):
        mock_engine.list_agents.return_value = []
        result = await service.list()
        assert result == []


class TestAgentServiceUpdate:
    """Tests for AgentService.update."""

    @pytest.mark.asyncio
    async def test_updates_agent(self, service, mock_engine, any_uuid):
        mock_engine.update_agent.return_value = {
            "id": any_uuid,
            "name": "Updated Agent",
            "provider": "openai",
            "model": "gpt-4",
        }
        patch = AgentPatch(name="Updated Agent")
        result = await service.update(any_uuid, patch)
        assert result is not None
        assert result.name == "Updated Agent"

    @pytest.mark.asyncio
    async def test_update_returns_none_when_empty(self, service, mock_engine):
        mock_engine.update_agent.return_value = {}
        patch = AgentPatch(name="Updated")
        result = await service.update("nonexistent", patch)
        assert result is None


class TestAgentServiceDelete:
    """Tests for AgentService.delete."""

    @pytest.mark.asyncio
    async def test_deletes_agent(self, service, mock_engine, any_uuid):
        await service.delete(any_uuid)
        mock_engine.delete_agent.assert_awaited_once_with(any_uuid)
