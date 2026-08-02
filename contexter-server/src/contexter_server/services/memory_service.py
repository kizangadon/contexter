"""Domain service for Memory aggregate operations."""

import asyncio

import structlog

from contexter_server.core.bridge import StorageEngine
from contexter_server.models.memory import Memory, MemoryCreate, MemoryPatch
from contexter_server.models.search import SearchQuery, SearchResult, SearchResponse

logger = structlog.get_logger(__name__)


class MemoryService:
    """Domain service for Memory aggregate operations."""

    def __init__(self, engine: StorageEngine) -> None:
        self._engine = engine

    async def create(self, data: MemoryCreate) -> Memory:
        payload = data.model_dump(mode="json")
        # The engine requires a semantic category (memoryType); the server's
        # domain contract is role-based. Default to the domain model's existing
        # default ("fact") until a role-specific mapping is specified.
        payload.setdefault("memory_type", "fact")
        raw = await self._engine.create_memory(payload)
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
        # Translate the domain query vocabulary to the engine's search query
        # vocabulary: query -> keywords, type -> memory_type. The bridge maps
        # the latter to the engine's camelCase contract (memoryType).
        if "query" in query_dict:
            query_dict["keywords"] = query_dict.pop("query")
        if "type" in query_dict:
            query_dict["memory_type"] = query_dict.pop("type")

        # Translate pagination from page/limit to limit/offset for the bridge
        bridge_limit = query.limit
        bridge_offset = (query.page - 1) * query.limit

        # Gather search and count concurrently
        raw_results, total_raw = await asyncio.gather(
            self._engine.search_memories(query_dict, limit=bridge_limit, offset=bridge_offset),
            self._engine.count_memories(query_dict),
            return_exceptions=True,
        )

        # Unpack the gathered calls, handling failures explicitly:
        # - A failed results call is a failed search: propagate the error
        #   rather than silently returning an empty page (EC-STF-001/002).
        if isinstance(raw_results, Exception):
            raise raw_results
        memory_results = raw_results

        # - A failed count call must never be silently reported as total=0
        #   (REQ-STF-001): log it explicitly and surface -1 as a
        #   distinguishing signal while still returning the results page.
        if isinstance(total_raw, Exception) or not isinstance(total_raw, int):
            logger.error(
                "search_count_failed",
                error_type=type(total_raw).__name__,
                error=str(total_raw),
            )
            total = -1
        else:
            total = total_raw

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
