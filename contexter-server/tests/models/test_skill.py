"""Tests for skill Pydantic models.

These tests pin the domain model to the engine's serde contract
(``contexter-core/src/models/skill.rs``): the engine calls the domain
``type`` field ``category``, stores ``version`` as a ``u32`` (coerced to the
domain's string form), emits ``filePath`` in camelCase, and renders
timestamps as ``createdAt``/``updatedAt``.
"""

import uuid
from datetime import datetime

import pytest
from pydantic import ValidationError

from contexter_server.models.skill import Skill, SkillCreate, SkillPatch


class TestSkillModel:
    """Skill model validation and serialization tests."""

    def test_skill_defaults(self):
        """Skill should auto-generate id and timestamps."""
        skill = Skill(name="test-skill", type="memory")
        assert isinstance(skill.id, uuid.UUID)
        assert isinstance(skill.created_at, datetime)
        assert isinstance(skill.updated_at, datetime)
        assert skill.enabled is True
        assert skill.parameters == {}
        assert skill.version == "1"
        assert skill.file_path is None

    def test_skill_minimal(self):
        """Skill with only required fields."""
        skill = Skill(name="search-skill", type="search")
        assert skill.name == "search-skill"
        assert skill.type == "search"

    def test_skill_with_all_fields(self):
        """Skill with all fields populated."""
        skill = Skill(
            name="advanced-reasoning",
            description="Complex chain-of-thought reasoning",
            type="reasoning",
            version="1.2.0",
            parameters={"max_steps": 5},
            enabled=False,
            file_path="/path/to/skill.py",
        )
        assert skill.description == "Complex chain-of-thought reasoning"
        assert skill.version == "1.2.0"
        assert skill.parameters == {"max_steps": 5}
        assert skill.enabled is False
        assert skill.file_path == "/path/to/skill.py"

    def test_skill_accepts_engine_category(self):
        """Engine ``category`` input key must populate the domain ``type``."""
        skill = Skill(name="engine-skill", category="search")
        assert skill.type == "search"

    def test_skill_accepts_engine_file_path(self):
        """Engine camelCase ``filePath`` input key must populate ``file_path``."""
        skill = Skill(name="engine-skill", type="search", filePath="/x/y.py")
        assert skill.file_path == "/x/y.py"

    def test_skill_version_int_coercion(self):
        """Engine u32 version must coerce to the domain string form."""
        skill = Skill(name="engine-skill", type="search", version=7)
        assert skill.version == "7"

    def test_skill_name_min_length(self):
        """Name must be at least 1 character."""
        with pytest.raises(ValidationError):
            Skill(name="", type="memory")

    def test_skill_name_max_length(self):
        """Name must not exceed 256 characters."""
        with pytest.raises(ValidationError):
            Skill(name="x" * 257, type="memory")

    def test_skill_description_max_length(self):
        """Description must not exceed 1024 characters."""
        with pytest.raises(ValidationError):
            Skill(name="n", type="t", description="x" * 1025)

    def test_skill_parses_real_engine_payload(self):
        """A real engine Skill payload (camelCase) must validate directly."""
        raw = {
            "id": "00000000-0000-0000-0000-000000000001",
            "name": "engine-skill",
            "description": "Built by the engine",
            "category": "memory",
            "version": 4,
            "filePath": "/tmp/skills/engine-skill.py",
            "createdAt": "2026-07-25T10:00:00Z",
            "updatedAt": "2026-07-25T10:05:00Z",
        }
        skill = Skill.model_validate(raw)
        assert skill.name == "engine-skill"
        assert skill.type == "memory"
        assert skill.version == "4"
        assert skill.file_path == "/tmp/skills/engine-skill.py"
        assert skill.created_at.isoformat() == "2026-07-25T10:00:00+00:00"

    def test_skill_accepts_snake_case_engine_payload(self):
        """Engine payloads with snake_case keys must validate too (mock parity)."""
        raw = {
            "id": "00000000-0000-0000-0000-000000000001",
            "name": "snake-skill",
            "type": "search",
            "version": "1.0.0",
            "parameters": {},
            "enabled": True,
            "created_at": "2026-07-25T10:00:00Z",
            "updated_at": "2026-07-25T10:00:00Z",
        }
        skill = Skill.model_validate(raw)
        assert skill.type == "search"
        assert skill.version == "1.0.0"

    def test_skill_serialization_roundtrip(self):
        """Skill should serialize and deserialize."""
        skill = Skill(name="test", type="custom", version="2", file_path="/a/b.py")
        data = skill.model_dump()
        restored = Skill.model_validate(data)
        assert restored.name == skill.name
        assert restored.type == skill.type
        assert restored.version == "2"
        assert restored.file_path == "/a/b.py"


class TestSkillCreateModel:
    """SkillCreate validation tests."""

    def test_skill_create_valid(self):
        """SkillCreate with valid data."""
        data = SkillCreate(name="new-skill", type="reasoning")
        assert data.enabled is True
        assert data.version == "1"

    def test_skill_create_disabled(self):
        """SkillCreate with enabled=False."""
        data = SkillCreate(name="disabled-skill", type="memory", enabled=False)
        assert data.enabled is False

    def test_skill_create_accepts_category(self):
        """SkillCreate must accept the engine ``category`` input key."""
        data = SkillCreate(name="new-skill", category="search")
        assert data.type == "search"

    def test_skill_create_requires_type(self):
        """SkillCreate without a type must fail validation."""
        with pytest.raises(ValidationError):
            SkillCreate(name="new-skill")


class TestSkillPatchModel:
    """SkillPatch validation tests."""

    def test_skill_patch_empty(self):
        """SkillPatch should allow empty patch."""
        patch = SkillPatch()
        assert patch.name is None

    def test_skill_patch_partial(self):
        """SkillPatch with some fields."""
        patch = SkillPatch(enabled=False)
        assert patch.enabled is False
        assert patch.name is None

    def test_skill_patch_accepts_category(self):
        """SkillPatch must accept the engine ``category`` input key."""
        patch = SkillPatch(category="search")
        assert patch.type == "search"
