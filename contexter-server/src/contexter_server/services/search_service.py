"""Domain service for cross-entity search operations."""

import asyncio

from contexter_server.core.bridge import StorageEngine
from contexter_server.models.search import SearchQuery, SearchResult, SearchResponse


def _unwrap_results(data: list | Exception | None) -> list:
    """Return the list if *data* is a list, otherwise an empty list."""
    return data if isinstance(data, list) else []


class SearchService:
    """Domain service for cross-entity search across memories and sessions."""

    def __init__(self, engine: StorageEngine) -> None:
        self._engine = engine

    async def search(self, query: SearchQuery) -> SearchResponse:
        """Execute a cross-entity search across memories and sessions."""
        query_dict = query.model_dump(exclude_none=True)

        # Translate pagination from page/limit to limit/offset for the bridge
        bridge_limit = query.limit
        bridge_offset = (query.page - 1) * query.limit

        # Gather memory search and session list concurrently when project filter given
        project = query_dict.get("project")
        if project:
            raw_memory_results, sessions = await asyncio.gather(
                self._engine.search_memories(query_dict, limit=bridge_limit, offset=bridge_offset),
                self._engine.list_sessions({"project": project}, limit=bridge_limit, offset=bridge_offset),
                return_exceptions=True,
            )
        else:
            raw_memory_results = await self._engine.search_memories(query_dict, limit=bridge_limit, offset=bridge_offset)
            sessions = []

        memory_results_list = _unwrap_results(raw_memory_results)
        session_results_list = _unwrap_results(sessions)

        results: list[SearchResult] = []
        for r in memory_results_list:
            results.append(
                SearchResult(
                    id=r.get("id", ""),
                    type="memory",
                    score=r.get("score", 0.0),
                    data=r,
                    snippet=r.get("content", "")[:200] if r.get("content") else None,
                )
            )

        for s in session_results_list:
            results.append(
                SearchResult(
                    id=s.get("id", ""),
                    type="session",
                    score=0.5,
                    data=s,
                    snippet=s.get("name") or s.get("project", ""),
                )
            )

        # Sort by score descending, take page slice
        results.sort(key=lambda x: x.score, reverse=True)
        total = len(results)
        start = (query.page - 1) * query.limit
        end = start + query.limit
        page_results = results[start:end]

        return SearchResponse(
            results=page_results,
            total=total,
            page=query.page,
            limit=query.limit,
        )
