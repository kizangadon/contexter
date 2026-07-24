"""Contexter async bridge — wraps Rust Engine via asyncio.to_thread + ThreadPoolExecutor."""

import asyncio
import json
from concurrent.futures import ThreadPoolExecutor
from typing import Any, Optional
from contexter import Engine as _SyncEngine

# Memories whose serialised JSON exceeds this size (in bytes) use a direct
# PyBytes path for the content field to avoid double JSON encoding overhead.
_MAX_MEMORY_JSON_SIZE = 102_400  # 100 KB


class Engine:
    """Async wrapper around the Rust Contexter Engine.

    All database operations run in a configurable ThreadPoolExecutor to
    avoid blocking the async event loop. JSON serialisation/deserialisation
    happens at the boundary for normal payloads; large memory content
    (>100 KB) is passed as raw PyBytes to avoid the double-encoding overhead.
    """

    def __init__(self, path: str, max_workers: int = 4):
        if max_workers <= 0:
            max_workers = 4
        self._max_workers = max_workers
        self._pool = ThreadPoolExecutor(max_workers=max_workers)
        self._engine = _SyncEngine.open(path)

    @classmethod
    def open(cls, path: str, max_workers: int = 4) -> "Engine":
        return cls(path, max_workers=max_workers)

    async def _run(self, method, *args):
        """Run a synchronous method in the thread pool."""
        loop = asyncio.get_event_loop()
        return await loop.run_in_executor(self._pool, method, *args)

    # -----------------------------------------------------------------------
    # Session methods
    # -----------------------------------------------------------------------

    async def create_session(self, session: dict) -> dict:
        result = await self._run(self._engine.create_session, json.dumps(session))
        return json.loads(result)

    async def get_session(self, id: str) -> Optional[dict]:
        result = await self._run(self._engine.get_session, id)
        return json.loads(result) if result else None

    async def list_sessions(self, filter: Optional[dict] = None) -> list[dict]:
        filter_json = json.dumps(filter) if filter else "{}"
        result = await self._run(self._engine.list_sessions, filter_json)
        return json.loads(result)

    async def update_session(self, id: str, patch: dict) -> Optional[dict]:
        result = await self._run(self._engine.update_session, id, json.dumps(patch))
        return json.loads(result) if result else None

    async def delete_session(self, id: str) -> None:
        await self._run(self._engine.delete_session, id)

    async def count_sessions(self, filter: Optional[dict] = None) -> int:
        filter_json = json.dumps(filter) if filter else "{}"
        return await self._run(self._engine.count_sessions, filter_json)

    # -----------------------------------------------------------------------
    # Memory methods
    # -----------------------------------------------------------------------

    async def create_memory(self, memory: dict) -> dict:
        content = memory.get("content", "")
        if len(content) > _MAX_MEMORY_JSON_SIZE:
            meta = {k: v for k, v in memory.items() if k != "content"}
            result = await self._run(
                self._engine.create_memory_bytes,
                json.dumps(meta),
                content.encode("utf-8"),
            )
        else:
            result = await self._run(self._engine.create_memory, json.dumps(memory))
        return json.loads(result)

    async def get_memory(self, id: str) -> Optional[dict]:
        result = await self._run(self._engine.get_memory, id)
        return json.loads(result) if result else None

    async def search_memories(self, query: dict) -> list[dict]:
        result = await self._run(self._engine.search_memories, json.dumps(query))
        return json.loads(result)

    async def update_memory(self, id: str, patch: dict) -> Optional[dict]:
        patch_content = patch.get("content", "")
        if patch_content and len(patch_content) > _MAX_MEMORY_JSON_SIZE:
            meta = {k: v for k, v in patch.items() if k != "content"}
            result = await self._run(
                self._engine.update_memory_bytes,
                id,
                json.dumps(meta),
                patch_content.encode("utf-8"),
            )
        else:
            result = await self._run(
                self._engine.update_memory, id, json.dumps(patch)
            )
        return json.loads(result) if result else None

    async def delete_memory(self, id: str) -> None:
        await self._run(self._engine.delete_memory, id)

    async def count_memories(self, query: dict) -> int:
        result = await self._run(self._engine.count_memories, json.dumps(query))
        return result

    # -----------------------------------------------------------------------
    # Agent methods
    # -----------------------------------------------------------------------

    async def create_agent(self, agent: dict) -> dict:
        result = await self._run(self._engine.create_agent, json.dumps(agent))
        return json.loads(result)

    async def get_agent(self, id: str) -> Optional[dict]:
        result = await self._run(self._engine.get_agent, id)
        return json.loads(result) if result else None

    async def list_agents(self, filter: Optional[dict] = None) -> list[dict]:
        filter_json = json.dumps(filter) if filter else "{}"
        result = await self._run(self._engine.list_agents, filter_json)
        return json.loads(result)

    async def update_agent(self, id: str, patch: dict) -> Optional[dict]:
        result = await self._run(self._engine.update_agent, id, json.dumps(patch))
        return json.loads(result) if result else None

    async def delete_agent(self, id: str) -> None:
        await self._run(self._engine.delete_agent, id)

    # -----------------------------------------------------------------------
    # Skill methods
    # -----------------------------------------------------------------------

    async def create_skill(self, skill: dict) -> dict:
        result = await self._run(self._engine.create_skill, json.dumps(skill))
        return json.loads(result)

    async def get_skill(self, id: str) -> Optional[dict]:
        result = await self._run(self._engine.get_skill, id)
        return json.loads(result) if result else None

    async def list_skills(self, filter: Optional[dict] = None) -> list[dict]:
        filter_json = json.dumps(filter) if filter else "{}"
        result = await self._run(self._engine.list_skills, filter_json)
        return json.loads(result)

    async def update_skill(self, id: str, patch: dict) -> Optional[dict]:
        result = await self._run(self._engine.update_skill, id, json.dumps(patch))
        return json.loads(result) if result else None

    async def delete_skill(self, id: str) -> None:
        await self._run(self._engine.delete_skill, id)

    # -----------------------------------------------------------------------
    # Settings
    # -----------------------------------------------------------------------

    async def set_setting(self, key: str, value: str) -> None:
        await self._run(self._engine.set_setting, key, value)

    async def get_setting(self, key: str) -> Optional[str]:
        return await self._run(self._engine.get_setting, key)

    # -----------------------------------------------------------------------
    # Audit
    # -----------------------------------------------------------------------

    async def log_audit(self, entry: dict) -> None:
        await self._run(self._engine.log_audit, json.dumps(entry))

    async def query_audit(self, filter: dict) -> list[dict]:
        result = await self._run(self._engine.query_audit, json.dumps(filter))
        return json.loads(result)

    # -----------------------------------------------------------------------
    # Maintenance
    # -----------------------------------------------------------------------

    async def flush(self) -> None:
        await self._run(self._engine.flush)

    async def checkpoint(self) -> int:
        return await self._run(self._engine.checkpoint)

    async def storage_size(self) -> dict:
        result = await self._run(self._engine.storage_size)
        return json.loads(result)

    async def status(self) -> dict:
        result = await self._run(self._engine.status)
        return json.loads(result)

    async def clear_cache(self) -> None:
        await self._run(self._engine.clear_cache)

    async def cache_telemetry(self) -> dict:
        result = await self._run(self._engine.cache_telemetry)
        return json.loads(result)

    async def clear_cache_type(self, entity_type: str) -> None:
        await self._run(self._engine.clear_cache_type, entity_type)