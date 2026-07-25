"""Domain service for data export operations."""

import asyncio
import json
from collections import OrderedDict
from collections.abc import Coroutine
from datetime import datetime, timezone
from typing import Any
from uuid import uuid4

from contexter_server.core.bridge import StorageEngine
from contexter_server.models.export import ExportRequest, ExportStatus

_ENTITY_TO_CORO = {
    "sessions": "list_sessions",
    "memories": "search_memories",
    "agents": "list_agents",
    "skills": "list_skills",
}

_ALL_ENTITIES = ["sessions", "memories", "agents", "skills"]


class ExportService:
    """Domain service for export request lifecycle management.

    Export statuses and data are persisted to the StorageEngine bridge via
    ``set_setting``/``get_setting`` and cached in an in-memory LRU cache
    (``OrderedDict`` with a maximum of 100 entries) for fast reads.
    """

    _MAX_CACHE_SIZE = 100

    def __init__(self, engine: StorageEngine) -> None:
        self._engine = engine
        # In-memory LRU caches: OrderedDict maintains insertion/access order.
        self._cache: OrderedDict[str, ExportStatus] = OrderedDict()
        self._data_cache: OrderedDict[str, dict] = OrderedDict()

    # ------------------------------------------------------------------
    # Persistence helpers
    # ------------------------------------------------------------------

    def _status_key(self, export_id: str) -> str:
        return f"export_status:{export_id}"

    def _data_key(self, export_id: str) -> str:
        return f"export_data:{export_id}"

    async def _persist_status(self, export_id: str, status: ExportStatus) -> None:
        """Persist an export status to the bridge as JSON."""
        raw = status.model_dump(mode="json", by_alias=True)
        await self._engine.set_setting(self._status_key(export_id), json.dumps(raw))

    async def _persist_data(self, export_id: str, data: dict) -> None:
        """Persist export data to the bridge as JSON."""
        await self._engine.set_setting(self._data_key(export_id), json.dumps(data))

    def _cache_put_status(self, key: str, status: ExportStatus) -> None:
        """Insert into LRU cache, evicting oldest if at capacity."""
        if key in self._cache:
            self._cache.move_to_end(key)
        self._cache[key] = status
        while len(self._cache) > self._MAX_CACHE_SIZE:
            self._cache.popitem(last=False)

    def _cache_put_data(self, key: str, data: dict) -> None:
        """Insert into data LRU cache, evicting oldest if at capacity."""
        if key in self._data_cache:
            self._data_cache.move_to_end(key)
        self._data_cache[key] = data
        while len(self._data_cache) > self._MAX_CACHE_SIZE:
            self._data_cache.popitem(last=False)

    # ------------------------------------------------------------------
    # Public API
    # ------------------------------------------------------------------

    async def submit(self, request: ExportRequest) -> ExportStatus:
        """Submit a new export request and begin processing."""
        export_id = uuid4()
        status = ExportStatus(
            id=export_id,
            status="in_progress",
            progress=0.0,
            format=request.format,
            created_at=datetime.now(timezone.utc),
        )

        # Collect data from bridge — all calls are independent, gather them
        entities = request.entities if request.entities else _ALL_ENTITIES

        bridge_coros: list[tuple[str, Coroutine[Any, Any, Any]]] = []
        for entity in entities:
            if entity == "sessions":
                coro = self._engine.list_sessions({}, limit=10_000)
            elif entity == "memories":
                coro = self._engine.search_memories({}, limit=10_000)
            elif entity == "agents":
                coro = self._engine.list_agents({}, limit=10_000)
            elif entity == "skills":
                coro = self._engine.list_skills({}, limit=10_000)
            else:
                continue
            bridge_coros.append((entity, coro))

        # Gather all bridge calls concurrently
        results = await asyncio.gather(
            *[c for _, c in bridge_coros],
            return_exceptions=True,
        )

        export_data: dict[str, list[dict]] = {}
        for (entity, _), result in zip(bridge_coros, results):
            if isinstance(result, list):
                export_data[entity] = result
            else:
                export_data[entity] = []

        status.progress = 1.0
        status.status = "completed"
        status.completed_at = datetime.now(timezone.utc)

        # Store export status and data in cache + persist to bridge
        export_id_str = str(export_id)
        self._cache_put_status(export_id_str, status)
        self._cache_put_data(export_id_str, export_data)

        # Persist to bridge for durability
        await self._persist_status(export_id_str, status)
        await self._persist_data(export_id_str, export_data)

        return status

    async def get_status(self, id: str) -> ExportStatus | None:
        """Get the status of an export by ID.

        Checks the in-memory LRU cache first; falls back to the bridge.
        """
        # Check in-memory cache first
        status = self._cache.get(id)
        if status is not None:
            self._cache.move_to_end(id)
            return status

        # Fall back to bridge
        raw = await self._engine.get_setting(self._status_key(id))
        if raw is None:
            return None

        parsed = json.loads(raw)
        status = ExportStatus.model_validate(parsed)
        self._cache_put_status(id, status)
        return status

    async def download(self, id: str) -> dict | None:
        """Download the exported data for a completed export.

        Checks the in-memory LRU cache first; falls back to the bridge.
        """
        # Check in-memory cache first
        data = self._data_cache.get(id)
        if data is not None:
            self._data_cache.move_to_end(id)
            return data

        # Fall back to bridge
        raw = await self._engine.get_setting(self._data_key(id))
        if raw is None:
            return None

        parsed = json.loads(raw)
        self._cache_put_data(id, parsed)
        return dict(parsed)

    async def history(self, limit: int = 20) -> list[ExportStatus]:
        """Get a list of recent export statuses from the cache."""
        statuses = list(self._cache.values())
        statuses.sort(key=lambda s: s.created_at, reverse=True)
        return statuses[:limit]
