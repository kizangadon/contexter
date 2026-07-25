"""Domain service for Skill aggregate operations."""

from contexter_server.core.bridge import StorageEngine
from contexter_server.models.skill import Skill, SkillCreate, SkillPatch


class SkillService:
    """Domain service for Skill aggregate operations."""

    def __init__(self, engine: StorageEngine) -> None:
        self._engine = engine

    async def create(self, data: SkillCreate) -> Skill:
        raw = await self._engine.create_skill(data.model_dump(mode="json"))
        return Skill.model_validate(raw)

    async def get(self, id: str) -> Skill | None:
        raw = await self._engine.get_skill(id)
        return Skill.model_validate(raw) if raw else None

    async def list(self, filter: dict | None = None) -> list[Skill]:
        raw_list = await self._engine.list_skills(filter)
        return [Skill.model_validate(r) for r in raw_list]

    async def update(self, id: str, patch: SkillPatch) -> Skill | None:
        raw = await self._engine.update_skill(id, patch.model_dump(exclude_none=True))
        return Skill.model_validate(raw) if raw else None

    async def delete(self, id: str) -> None:
        await self._engine.delete_skill(id)
