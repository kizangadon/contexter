"""Domain service for Agent aggregate operations."""

from contexter_server.core.bridge import StorageEngine
from contexter_server.models.agent import Agent, AgentCreate, AgentPatch


class AgentService:
    """Domain service for Agent aggregate operations."""

    def __init__(self, engine: StorageEngine) -> None:
        self._engine = engine

    async def create(self, data: AgentCreate) -> Agent:
        raw = await self._engine.create_agent(data.model_dump(mode="json"))
        return Agent.model_validate(raw)

    async def get(self, id: str) -> Agent | None:
        raw = await self._engine.get_agent(id)
        return Agent.model_validate(raw) if raw else None

    async def list(self, filter: dict | None = None) -> list[Agent]:
        raw_list = await self._engine.list_agents(filter)
        return [Agent.model_validate(r) for r in raw_list]

    async def update(self, id: str, patch: AgentPatch) -> Agent | None:
        raw = await self._engine.update_agent(id, patch.model_dump(exclude_none=True))
        return Agent.model_validate(raw) if raw else None

    async def delete(self, id: str) -> None:
        await self._engine.delete_agent(id)
