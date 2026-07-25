"""Tests for agent Pydantic models."""

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
        assert agent.tools == []
        assert agent.metadata == {}

    def test_agent_minimal(self):
        """Agent with only required fields."""
        agent = Agent(name="my-agent", provider="anthropic", model="claude-3")
        assert agent.name == "my-agent"
        assert agent.provider == "anthropic"
        assert agent.model == "claude-3"

    def test_agent_with_all_fields(self):
        """Agent with all fields populated."""
        agent = Agent(
            name="advanced-agent",
            provider="ollama",
            model="llama3",
            system_prompt="You are helpful.",
            temperature=0.5,
            max_tokens=4096,
            tools=["search", "code"],
            metadata={"version": "1.0"},
        )
        assert agent.system_prompt == "You are helpful."
        assert agent.temperature == 0.5
        assert agent.max_tokens == 4096
        assert agent.tools == ["search", "code"]

    def test_agent_name_min_length(self):
        """Name must be at least 1 character."""
        with pytest.raises(ValidationError):
            Agent(name="", provider="openai", model="gpt-4")

    def test_agent_name_max_length(self):
        """Name must not exceed 256 characters."""
        with pytest.raises(ValidationError):
            Agent(name="x" * 257, provider="openai", model="gpt-4")

    def test_agent_temperature_range(self):
        """Temperature must be between 0.0 and 2.0."""
        with pytest.raises(ValidationError):
            Agent(name="a", provider="o", model="m", temperature=-0.1)
        with pytest.raises(ValidationError):
            Agent(name="a", provider="o", model="m", temperature=2.1)

    def test_agent_max_tokens_positive(self):
        """max_tokens must be > 0."""
        with pytest.raises(ValidationError):
            Agent(name="a", provider="o", model="m", max_tokens=0)

    def test_agent_serialization_roundtrip(self):
        """Agent should serialize and deserialize."""
        agent = Agent(name="test", provider="openai", model="gpt-4")
        data = agent.model_dump()
        restored = Agent.model_validate(data)
        assert restored.name == agent.name
        assert restored.provider == agent.provider


class TestAgentCreateModel:
    """AgentCreate validation tests."""

    def test_agent_create_valid(self):
        """AgentCreate with valid data."""
        data = AgentCreate(name="new-agent", provider="openai", model="gpt-4")
        assert data.temperature == 0.7
        assert data.tools == []


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
