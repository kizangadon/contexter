"""Domain service for Memory aggregate operations."""

import asyncio

from contexter_server.core.bridge import StorageEngine
from contexter_server.models.memory import Memory, MemoryCreate, MemoryPatch
from contexter_server.models.search import SearchQuery, SearchResult, SearchResponse


class MemoryService:
    """Domain service for Memory aggregate operations."""

    def __init__(self, engine: StorageEngine) -> None:
        self._engine = engine

    async def create(self, data: MemoryCreate) -> Memory:
        raw = await self._engine.create_memory(data.model_dump(mode="json"))
        return Memory.model_validate(raw)

    async def get(self, id: str) -> Memory | None:
        raw = await self._engine.get_memory(id)
        return Memory.model_validate(raw) if raw else None

    async def list(self) -> list[Memory]:
        raw_list = await self._engine.search_memories({}, limit=100, offset=0)
        return [Memory.model_validate(r) for r in raw_list]

    async def update(self, id: str, patch: MemoryPatch) -> Memory | None:
        raw = await self._engine.update_memory(id, patch.model_dump(exclude_none=True))
        return Memory.model_validate(raw) if raw else None

    async def delete(self, id: str) -> None:
        await self._engine.delete_memory(id)

    async def search(self, query: SearchQuery) -> SearchResponse:
        """Search memories with the given query parameters."""
        query_dict = query.model_dump(exclude_none=True)

        # Translate pagination from page/limit to limit/offset for the bridge
        bridge_limit = query.limit
        bridge_offset = (query.page - 1) * query.limit

        # Gather search and count concurrently
        raw_results, total_raw = await asyncio.gather(
            self._engine.search_memories(query_dict, limit=bridge_limit, offset=bridge_offset),
            self._engine.count_memories(query_dict),
            return_exceptions=True,
        )

        # Unpack results, handling any exceptions gracefully
        memory_results = raw_results if isinstance(raw_results, list) else []
        total = total_raw if isinstance(total_raw, int) else 0

        results = [
            SearchResult(
                id=r.get("id", ""),
                type="memory",
                score=r.get("score", 0.0),
                data={k: v for k, v in r.items() if k != "embedding"},
                snippet=r.get("content", "")[:200] if r.get("content") else None,
            )
            for r in memory_results
        ]

        return SearchResponse(
            results=results,
            total=total,
            page=query.page,
            limit=query.limit,
        )
