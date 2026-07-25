"""Tests for skill Pydantic models."""

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
        )
        assert skill.description == "Complex chain-of-thought reasoning"
        assert skill.version == "1.2.0"
        assert skill.parameters == {"max_steps": 5}
        assert skill.enabled is False

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

    def test_skill_serialization_roundtrip(self):
        """Skill should serialize and deserialize."""
        skill = Skill(name="test", type="custom")
        data = skill.model_dump()
        restored = Skill.model_validate(data)
        assert restored.name == skill.name
        assert restored.type == skill.type


class TestSkillCreateModel:
    """SkillCreate validation tests."""

    def test_skill_create_valid(self):
        """SkillCreate with valid data."""
        data = SkillCreate(name="new-skill", type="reasoning")
        assert data.enabled is True

    def test_skill_create_disabled(self):
        """SkillCreate with enabled=False."""
        data = SkillCreate(name="disabled-skill", type="memory", enabled=False)
        assert data.enabled is False


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
