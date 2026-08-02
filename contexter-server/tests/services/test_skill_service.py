"""Tests for SkillService.

The service is the translation boundary between the domain models and the
Rust engine's serde contract. The engine calls the domain ``type`` field
``category`` and its ``SkillFilter`` has no ``type`` field — a raw
``{"type": ...}`` filter is silently dropped by serde, so the service MUST
enforce the domain filter itself.
"""

from unittest.mock import AsyncMock

import pytest

from contexter_server.models.skill import SkillCreate, SkillPatch
from contexter_server.services.skill_service import SkillService


@pytest.fixture
def mock_engine():
    engine = AsyncMock()
    return engine


@pytest.fixture
def service(mock_engine):
    return SkillService(mock_engine)


@pytest.fixture
def engine_skill(any_uuid: str) -> dict:
    """A realistic engine Skill payload (what contexter_core returns)."""
    return {
        "id": any_uuid,
        "name": "memory-skill",
        "description": "A memory skill",
        "category": "memory",
        "version": 1,
        "filePath": None,
        "createdAt": "2026-07-25T10:00:00Z",
        "updatedAt": "2026-07-25T10:00:00Z",
    }


class TestSkillServiceCreate:
    """Tests for SkillService.create."""

    @pytest.mark.asyncio
    async def test_creates_skill_with_engine_payload(self, service, mock_engine, engine_skill):
        """Create must send the engine's ``category`` field, not ``type``."""
        mock_engine.create_skill.return_value = engine_skill
        data = SkillCreate(name="memory-skill", type="memory", description="A memory skill")
        result = await service.create(data)
        assert str(result.id) == engine_skill["id"]
        assert result.name == "memory-skill"
        assert result.type == "memory"
        assert result.version == "1"

        payload = mock_engine.create_skill.call_args[0][0]
        assert payload["name"] == "memory-skill"
        assert payload["category"] == "memory"  # engine vocabulary
        assert payload["description"] == "A memory skill"
        assert "type" not in payload  # domain vocabulary never reaches the engine

    @pytest.mark.asyncio
    async def test_create_defaults_description_for_engine(self, service, mock_engine, engine_skill):
        """Engine requires a description string; None must default to empty."""
        mock_engine.create_skill.return_value = engine_skill
        data = SkillCreate(name="memory-skill", type="memory")
        await service.create(data)

        payload = mock_engine.create_skill.call_args[0][0]
        assert payload["description"] == ""

    @pytest.mark.asyncio
    async def test_create_sends_file_path(self, service, mock_engine, engine_skill):
        """file_path must be forwarded to the engine (bridge camelizes it)."""
        mock_engine.create_skill.return_value = engine_skill
        data = SkillCreate(name="memory-skill", type="memory", file_path="/tmp/skill.py")
        await service.create(data)

        payload = mock_engine.create_skill.call_args[0][0]
        assert payload["file_path"] == "/tmp/skill.py"


class TestSkillServiceGet:
    """Tests for SkillService.get."""

    @pytest.mark.asyncio
    async def test_gets_skill(self, service, mock_engine, engine_skill):
        """Get must map the engine's category back to the domain type."""
        mock_engine.get_skill.return_value = engine_skill
        result = await service.get(engine_skill["id"])
        assert result is not None
        assert result.name == "memory-skill"
        assert result.type == "memory"
        assert result.version == "1"

    @pytest.mark.asyncio
    async def test_get_returns_none_when_not_found(self, service, mock_engine):
        mock_engine.get_skill.return_value = None
        result = await service.get("nonexistent")
        assert result is None


class TestSkillServiceList:
    """Tests for SkillService.list."""

    @pytest.mark.asyncio
    async def test_lists_skills(self, service, mock_engine, engine_skill, any_uuid):
        sid2 = any_uuid.replace("000001", "000003")
        second = {
            "id": sid2,
            "name": "search-skill",
            "description": None,
            "category": "search",
            "version": 1,
            "filePath": None,
            "createdAt": "2026-07-25T10:00:00Z",
            "updatedAt": "2026-07-25T10:00:00Z",
        }
        mock_engine.list_skills.return_value = [engine_skill, second]
        result = await service.list()
        assert len(result) == 2
        assert result[0].type == "memory"
        assert result[1].type == "search"

    @pytest.mark.asyncio
    async def test_list_returns_empty(self, service, mock_engine):
        mock_engine.list_skills.return_value = []
        result = await service.list()
        assert result == []

    @pytest.mark.asyncio
    async def test_list_type_filter_translated_and_enforced(self, service, mock_engine, engine_skill, any_uuid):
        """The engine silently drops ``type`` from SkillFilter — the service must enforce it.

        The engine returns ALL skills when given a ``{"type": ...}`` filter
        (serde ignores the unknown key). The service must translate the filter
        to the engine's ``category`` vocabulary AND re-apply the domain filter
        so a silent drop never reaches callers.
        """
        sid2 = any_uuid.replace("000001", "000003")
        search_skill = {
            "id": sid2,
            "name": "search-skill",
            "description": None,
            "category": "search",
            "version": 1,
            "filePath": None,
            "createdAt": "2026-07-25T10:00:00Z",
            "updatedAt": "2026-07-25T10:00:00Z",
        }
        # Simulate the engine's silent drop: it returns BOTH skills.
        mock_engine.list_skills.return_value = [engine_skill, search_skill]

        result = await service.list({"type": "memory"})

        assert [s.name for s in result] == ["memory-skill"]
        # The engine filter must use the engine vocabulary.
        engine_filter = mock_engine.list_skills.call_args[0][0]
        assert engine_filter["category"] == "memory"
        assert "type" not in engine_filter

    @pytest.mark.asyncio
    async def test_list_type_filter_case_insensitive(self, service, mock_engine, engine_skill, any_uuid):
        """The domain type filter must match the engine's case-insensitive semantics."""
        sid2 = any_uuid.replace("000001", "000003")
        search_skill = dict(engine_skill, id=sid2, name="search-skill", category="search")
        mock_engine.list_skills.return_value = [engine_skill, search_skill]

        result = await service.list({"type": "MEMORY"})

        assert [s.name for s in result] == ["memory-skill"]

    @pytest.mark.asyncio
    async def test_list_without_filter_passes_none(self, service, mock_engine):
        mock_engine.list_skills.return_value = []
        result = await service.list()
        assert result == []
        mock_engine.list_skills.assert_awaited_once_with(None)


class TestSkillServiceUpdate:
    """Tests for SkillService.update."""

    @pytest.mark.asyncio
    async def test_updates_skill(self, service, mock_engine, engine_skill):
        """A name-only patch must translate to a name-only engine patch."""
        updated = dict(engine_skill, name="Updated Skill")
        mock_engine.update_skill.return_value = updated
        patch = SkillPatch(name="Updated Skill")
        result = await service.update(engine_skill["id"], patch)
        assert result is not None
        assert result.name == "Updated Skill"
        mock_engine.update_skill.assert_awaited_once_with(
            engine_skill["id"], {"name": "Updated Skill"}
        )

    @pytest.mark.asyncio
    async def test_update_translates_type_to_category(self, service, mock_engine, engine_skill):
        """A type patch must reach the engine as ``category``."""
        mock_engine.update_skill.return_value = dict(engine_skill)
        patch = SkillPatch(type="reasoning")
        await service.update(engine_skill["id"], patch)

        payload = mock_engine.update_skill.call_args[0][1]
        assert payload["category"] == "reasoning"
        assert "type" not in payload

    @pytest.mark.asyncio
    async def test_update_forwards_file_path(self, service, mock_engine, engine_skill):
        """A file_path patch must be forwarded (bridge camelizes it)."""
        mock_engine.update_skill.return_value = dict(engine_skill)
        patch = SkillPatch(file_path="/new/path.py")
        await service.update(engine_skill["id"], patch)

        payload = mock_engine.update_skill.call_args[0][1]
        assert payload["file_path"] == "/new/path.py"

    @pytest.mark.asyncio
    async def test_update_returns_none_when_empty(self, service, mock_engine):
        mock_engine.update_skill.return_value = {}
        patch = SkillPatch(name="Updated")
        result = await service.update("nonexistent", patch)
        assert result is None


class TestSkillServiceDelete:
    """Tests for SkillService.delete."""

    @pytest.mark.asyncio
    async def test_deletes_skill(self, service, mock_engine, any_uuid):
        await service.delete(any_uuid)
        mock_engine.delete_skill.assert_awaited_once_with(any_uuid)
