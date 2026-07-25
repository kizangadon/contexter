"""Tests for SkillService."""

from unittest.mock import AsyncMock

import pytest

from contexter_server.models.skill import Skill, SkillCreate, SkillPatch
from contexter_server.services.skill_service import SkillService


@pytest.fixture
def mock_engine():
    engine = AsyncMock()
    return engine


@pytest.fixture
def service(mock_engine):
    return SkillService(mock_engine)


class TestSkillServiceCreate:
    """Tests for SkillService.create."""

    @pytest.mark.asyncio
    async def test_creates_skill(self, service, mock_engine, any_uuid):
        mock_engine.create_skill.return_value = {
            "id": any_uuid,
            "name": "Test Skill",
            "type": "memory",
        }
        data = SkillCreate(name="Test Skill", type="memory")
        result = await service.create(data)
        assert str(result.id) == any_uuid
        assert result.name == "Test Skill"
        mock_engine.create_skill.assert_awaited_once()


class TestSkillServiceGet:
    """Tests for SkillService.get."""

    @pytest.mark.asyncio
    async def test_gets_skill(self, service, mock_engine, any_uuid):
        mock_engine.get_skill.return_value = {
            "id": any_uuid,
            "name": "Test Skill",
            "type": "memory",
        }
        result = await service.get(any_uuid)
        assert result is not None
        assert result.name == "Test Skill"

    @pytest.mark.asyncio
    async def test_get_returns_none_when_not_found(self, service, mock_engine):
        mock_engine.get_skill.return_value = None
        result = await service.get("nonexistent")
        assert result is None


class TestSkillServiceList:
    """Tests for SkillService.list."""

    @pytest.mark.asyncio
    async def test_lists_skills(self, service, mock_engine, any_uuid):
        sid1 = any_uuid.replace("000001", "000002")
        sid2 = any_uuid.replace("000001", "000003")
        mock_engine.list_skills.return_value = [
            {"id": sid1, "name": "Skill 1", "type": "memory"},
            {"id": sid2, "name": "Skill 2", "type": "search"},
        ]
        result = await service.list()
        assert len(result) == 2
        assert result[0].name == "Skill 1"

    @pytest.mark.asyncio
    async def test_list_returns_empty(self, service, mock_engine):
        mock_engine.list_skills.return_value = []
        result = await service.list()
        assert result == []


class TestSkillServiceUpdate:
    """Tests for SkillService.update."""

    @pytest.mark.asyncio
    async def test_updates_skill(self, service, mock_engine, any_uuid):
        mock_engine.update_skill.return_value = {
            "id": any_uuid,
            "name": "Updated Skill",
            "type": "reasoning",
        }
        patch = SkillPatch(name="Updated Skill", type="reasoning")
        result = await service.update(any_uuid, patch)
        assert result is not None
        assert result.name == "Updated Skill"

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
