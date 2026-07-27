"""Tests for memory Pydantic models."""

import uuid
from datetime import datetime, timezone

import pytest
from pydantic import ValidationError

from contexter_server.models.memory import Memory, MemoryCreate, MemoryPatch


class TestMemoryModel:
    """Memory model validation and serialization tests."""

    def test_memory_defaults(self):
        """Memory should auto-generate id and timestamp."""
        mem = Memory(
            session_id=uuid.uuid4(),
            agent_id=uuid.uuid4(),
            role="user",
            content="Hello",
        )
        assert isinstance(mem.id, uuid.UUID)
        assert isinstance(mem.created_at, datetime)
        assert mem.metadata == {}
        assert mem.tokens is None

    def test_memory_minimal(self):
        """Memory with only required fields."""
        session_id = uuid.uuid4()
        agent_id = uuid.uuid4()
        mem = Memory(session_id=session_id, agent_id=agent_id, role="assistant", content="Hi!")
        assert mem.session_id == session_id
        assert mem.agent_id == agent_id
        assert mem.role == "assistant"
        assert mem.content == "Hi!"

    def test_role_default_is_system(self):
        """Memory role should default to 'system' when not specified."""
        mem = Memory(
            session_id=uuid.uuid4(),
            agent_id=uuid.uuid4(),
            content="Default role test",
        )
        assert mem.role == "system"

    def test_memory_with_all_fields(self):
        """Memory with all fields."""
        mem = Memory(
            session_id=uuid.uuid4(),
            agent_id=uuid.uuid4(),
            role="tool",
            content='{"result": "ok"}',
            tokens=150,
            tokenizer="cl100k_base",
            model="gpt-4",
            metadata={"cost": 0.01},
        )
        assert mem.tokens == 150
        assert mem.tokenizer == "cl100k_base"
        assert mem.model == "gpt-4"
        assert mem.metadata == {"cost": 0.01}

    def test_memory_serialization_roundtrip(self):
        """Memory should serialize and deserialize."""
        mem = Memory(
            session_id=uuid.uuid4(),
            agent_id=uuid.uuid4(),
            role="system",
            content="You are helpful.",
        )
        data = mem.model_dump()
        restored = Memory.model_validate(data)
        assert restored.id == mem.id
        assert restored.content == mem.content

    def test_memory_json_roundtrip(self):
        """Memory should serialize to JSON and back."""
        mem = Memory(
            session_id=uuid.uuid4(),
            agent_id=uuid.uuid4(),
            role="user",
            content="Hello world",
        )
        json_str = mem.model_dump_json()
        restored = Memory.model_validate_json(json_str)
        assert restored.content == "Hello world"

    def test_agent_id_optional_none(self):
        """Memory with no agent_id defaults to None."""
        mem = Memory(
            session_id=uuid.uuid4(),
            content="test memory without agent_id",
        )
        assert mem.agent_id is None

    def test_role_explicit_none(self):
        """Memory with explicit role=None should be None, not 'system'."""
        mem = Memory(
            session_id=uuid.uuid4(),
            agent_id=uuid.uuid4(),
            content="explicit null role",
            role=None,
        )
        assert mem.role is None

    def test_embedding_excluded_from_serialization(self):
        """model_dump and model_dump_json should not include embedding."""
        mem = Memory(
            session_id=uuid.uuid4(),
            agent_id=uuid.uuid4(),
            content="test",
            embedding=[0.1, 0.2, 0.3],
        )
        data = mem.model_dump()
        assert 'embedding' not in data
        json_str = mem.model_dump_json()
        assert 'embedding' not in json_str

    def test_naive_datetime_coerced_to_utc(self):
        """A naive datetime should be coerced to UTC-aware."""
        mem = Memory(
            session_id=uuid.uuid4(),
            agent_id=uuid.uuid4(),
            content="test",
            created_at=datetime(2024, 1, 1, 12, 0, 0),  # naive
        )
        assert mem.created_at.tzinfo is not None
        assert mem.created_at.tzinfo.utcoffset(mem.created_at) == timezone.utc.utcoffset(datetime(2024, 1, 1, 12, 0, 0, tzinfo=timezone.utc))

    def test_camelcase_alias_deserialization(self):
        """Memory should accept camelCase JSON fields via validation_alias."""
        json_data = (
            '{"id": "00000000-0000-0000-0000-000000000001", '
            '"sessionId": "00000000-0000-0000-0000-000000000002", '
            '"agentId": "00000000-0000-0000-0000-000000000003", '
            '"content": "alias test"}'
        )
        mem = Memory.model_validate_json(json_data)
        assert mem.session_id == uuid.UUID("00000000-0000-0000-0000-000000000002")
        assert mem.agent_id == uuid.UUID("00000000-0000-0000-0000-000000000003")
        assert mem.content == "alias test"


class TestMemoryCreateModel:
    """MemoryCreate validation tests."""

    def test_memory_create_valid(self):
        """MemoryCreate should accept valid data."""
        data = MemoryCreate(
            session_id=uuid.uuid4(),
            agent_id=uuid.uuid4(),
            role="user",
            content="Test",
        )
        assert data.role == "user"
        assert data.content == "Test"
        assert data.metadata == {}

    def test_memory_create_with_tokens(self):
        """MemoryCreate with token and model fields."""
        data = MemoryCreate(
            session_id=uuid.uuid4(),
            agent_id=uuid.uuid4(),
            role="assistant",
            content="Response",
            tokens=50,
            tokenizer="cl100k_base",
            model="gpt-4",
        )
        assert data.tokens == 50
        assert data.model == "gpt-4"


class TestMemoryPatchModel:
    """MemoryPatch validation tests."""

    def test_memory_patch_empty(self):
        """MemoryPatch should allow empty patch."""
        patch = MemoryPatch()
        assert patch.content is None
        assert patch.tokens is None

    def test_memory_patch_content(self):
        """MemoryPatch with content update."""
        patch = MemoryPatch(content="Updated content")
        assert patch.content == "Updated content"
        assert patch.tokens is None

    def test_memory_patch_metadata(self):
        """MemoryPatch with metadata update."""
        patch = MemoryPatch(metadata={"edited": True})
        assert patch.metadata == {"edited": True}
