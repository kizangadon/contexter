"""Tests for agent Pydantic models.

These tests pin the domain model to the engine's serde contract
(``contexter-core/src/models/agent.rs``): the engine emits camelCase JSON
(``createdAt``, ``updatedAt``, ``type``), ``status`` is the lowercase
camelCase rendering of ``AgentStatus`` (``"active"``/``"inactive"``), and
``provider``/``model`` are *not* engine fields — they are resolved from the
engine's opaque ``config`` blob at the service boundary.
"""

import uuid
from datetime import datetime

import pytest
from pydantic import ValidationError

from contexter_server.models.agent import Agent, AgentCreate, AgentPatch


class TestAgentModel:
    """Agent model validation and serialization tests."""

    def test_agent_defaults(self):
        """Agent should auto-generate id and timestamps."""
        agent = Agent(name="test-agent", provider="openai", model="gpt-4")
        assert isinstance(agent.id, uuid.UUID)
        assert isinstance(agent.created_at, datetime)
        assert isinstance(agent.updated_at, datetime)
        assert agent.temperature == 0.7
        assert agent.capabilities == []
        assert agent.metadata == {}

    def test_agent_minimal(self):
        """Agent with only required fields (provider/model are optional)."""
        agent = Agent(name="my-agent")
        assert agent.name == "my-agent"
        assert agent.provider is None
        assert agent.model is None
        assert agent.type == "general"
        assert agent.status == "active"
        assert agent.version == 1

    def test_agent_with_all_fields(self):
        """Agent with all fields populated."""
        agent = Agent(
            name="advanced-agent",
            provider="ollama",
            model="llama3",
            system_prompt="You are helpful.",
            temperature=0.5,
            max_tokens=4096,
            capabilities=["search", "code"],
            metadata={"version": "1.0"},
        )
        assert agent.system_prompt == "You are helpful."
        assert agent.temperature == 0.5
        assert agent.max_tokens == 4096
        assert agent.capabilities == ["search", "code"]

    def test_agent_accepts_legacy_tools_key(self):
        """Legacy ``tools`` input key must populate ``capabilities``."""
        agent = Agent(name="legacy", tools=["search", "code"])
        assert agent.capabilities == ["search", "code"]

    def test_agent_name_min_length(self):
        """Name must be at least 1 character."""
        with pytest.raises(ValidationError):
            Agent(name="")

    def test_agent_name_max_length(self):
        """Name must not exceed 256 characters."""
        with pytest.raises(ValidationError):
            Agent(name="x" * 257)

    def test_agent_temperature_range(self):
        """Temperature must be between 0.0 and 2.0."""
        with pytest.raises(ValidationError):
            Agent(name="a", temperature=-0.1)
        with pytest.raises(ValidationError):
            Agent(name="a", temperature=2.1)

    def test_agent_max_tokens_positive(self):
        """max_tokens must be > 0."""
        with pytest.raises(ValidationError):
            Agent(name="a", max_tokens=0)

    def test_agent_status_must_be_engine_vocabulary(self):
        """Status must be the engine's lowercase camelCase AgentStatus values."""
        with pytest.raises(ValidationError):
            Agent(name="a", status="bogus")
        with pytest.raises(ValidationError):
            Agent(name="a", status="Active")
        agent = Agent(name="a", status="inactive")
        assert agent.status == "inactive"

    def test_agent_parses_real_engine_payload(self):
        """A real engine Agent payload (camelCase) must validate directly."""
        raw = {
            "id": "00000000-0000-0000-0000-000000000001",
            "name": "engine-agent",
            "type": "coding-assistant",
            "description": "Built by the engine",
            "capabilities": ["code", "terminal"],
            "status": "active",
            "config": {"provider": "openai", "model": "gpt-4o"},
            "version": 3,
            "createdAt": "2026-07-25T10:00:00Z",
            "updatedAt": "2026-07-25T10:05:00Z",
        }
        agent = Agent.model_validate(raw)
        assert agent.name == "engine-agent"
        assert agent.type == "coding-assistant"
        assert agent.description == "Built by the engine"
        assert agent.capabilities == ["code", "terminal"]
        assert agent.status == "active"
        assert agent.version == 3
        assert agent.created_at.isoformat() == "2026-07-25T10:00:00+00:00"

    def test_agent_accepts_snake_case_engine_payload(self):
        """Engine payloads with snake_case keys must validate too (mock parity)."""
        raw = {
            "id": "00000000-0000-0000-0000-000000000001",
            "name": "snake-agent",
            "type": "chat",
            "description": None,
            "capabilities": [],
            "status": "active",
            "version": 1,
            "created_at": "2026-07-25T10:00:00Z",
            "updated_at": "2026-07-25T10:00:00Z",
        }
        agent = Agent.model_validate(raw)
        assert agent.type == "chat"
        assert agent.status == "active"
        assert agent.version == 1

    def test_agent_serialization_roundtrip(self):
        """Agent should serialize and deserialize."""
        agent = Agent(
            name="test",
            provider="openai",
            model="gpt-4",
            type="chat",
            capabilities=["search"],
        )
        data = agent.model_dump()
        restored = Agent.model_validate(data)
        assert restored.name == agent.name
        assert restored.provider == agent.provider
        assert restored.type == agent.type
        assert restored.capabilities == ["search"]


class TestAgentCreateModel:
    """AgentCreate validation tests."""

    def test_agent_create_valid(self):
        """AgentCreate with valid data."""
        data = AgentCreate(name="new-agent", provider="openai", model="gpt-4")
        assert data.temperature == 0.7
        assert data.capabilities == []
        assert data.type == "general"

    def test_agent_create_without_provider_model(self):
        """AgentCreate must not require fields the engine never sends."""
        data = AgentCreate(name="new-agent")
        assert data.provider is None
        assert data.model is None

    def test_agent_create_accepts_legacy_tools(self):
        """AgentCreate must accept the legacy ``tools`` input key."""
        data = AgentCreate(name="new-agent", tools=["search"])
        assert data.capabilities == ["search"]

    def test_agent_create_requires_name(self):
        """AgentCreate without a name must fail validation."""
        with pytest.raises(ValidationError):
            AgentCreate(name="")


class TestAgentPatchModel:
    """AgentPatch validation tests."""

    def test_agent_patch_empty(self):
        """AgentPatch should allow empty patch."""
        patch = AgentPatch()
        assert patch.name is None

    def test_agent_patch_partial(self):
        """AgentPatch with partial fields."""
        patch = AgentPatch(temperature=0.3)
        assert patch.temperature == 0.3
        assert patch.name is None

    def test_agent_patch_accepts_legacy_tools(self):
        """AgentPatch must accept the legacy ``tools`` input key."""
        patch = AgentPatch(tools=["search"])
        assert patch.capabilities == ["search"]
