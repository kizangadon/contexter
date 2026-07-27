"""Async bridge wrapping the Rust contexter_core Engine via asyncio.to_thread + ThreadPoolExecutor.

All database operations run in a configurable ThreadPoolExecutor to avoid
blocking the async event loop. JSON serialisation/deserialisation happens
at the boundary for normal payloads; large memory content (>=100 KB) is
passed as raw PyBytes to avoid double-encoding overhead.
"""

import asyncio
import json
import os
import time
from concurrent.futures import ThreadPoolExecutor

import structlog

from contexter_core import Engine as _SyncEngine

# Capture the raw class before any test patching so _run can validate
# method existence independently of a potentially-mocked instance.
_SYNC_ENGINE_CLASS = _SyncEngine

_LARGE_CONTENT_THRESHOLD = 102_400  # 100 KB

logger = structlog.get_logger(__name__)


def _truncated_args_summary(args: tuple, max_len: int = 200) -> str:
    """Build a truncated repr of *args* without constructing the full repr of
    large string/bytes elements.

    Unlike ``str(args)[:max_len]``, which calls ``repr()`` on every element
    and builds the entire string before slicing, this function truncates
    individual string/bytes arguments to avoid allocating large intermediate
    strings.  For a tuple containing a 100 KB string, the 100 KB+ repr is
    never materialised.
    """
    if not args:
        return "()"

    pieces: list[str] = []
    for arg in args:
        if isinstance(arg, str):
            if len(arg) <= max_len // 2:
                pieces.append(repr(arg))
            else:
                # Only construct repr of a short prefix — never the full string.
                snippet = arg[:max(0, max_len // 2 - 4)]
                pieces.append(repr(snippet)[:-1] + "...'")
        elif isinstance(arg, bytes):
            if len(arg) <= max_len // 2:
                pieces.append(repr(arg))
            else:
                snippet = arg[:max(0, max_len // 2 - 6)]
                pieces.append(repr(snippet)[:-1] + "...'")
        else:
            pieces.append(repr(arg))

    result = "(" + ", ".join(pieces)
    if len(args) == 1:
        result += ","
    result += ")"

    if len(result) > max_len:
        result = result[: max_len - 3] + "..."

    return result


class StorageEngine:
    """Async wrapper around the Rust Contexter Engine.

    All database operations are offloaded to a ThreadPoolExecutor so the
    calling async code never blocks on storage I/O.
    """

    def __init__(self, path: str, max_workers: int | None = None) -> None:
        # Expand leading ~/ to the user's home directory — the Rust Engine
        # does not perform tilde expansion, so ``"~/.contexter/"`` must be
        # resolved before it reaches RocksDB.
        expanded_path = os.path.expanduser(path)

        if max_workers is None:
            env_val = os.environ.get("CONtexTER_BRIDGE_POOL_SIZE", "")
            if env_val.strip():
                try:
                    max_workers = int(env_val)
                except (ValueError, TypeError):
                    max_workers = 8
            else:
                max_workers = 8
        if max_workers <= 0:
            max_workers = 8
        self._max_workers = max_workers
        self._pool = ThreadPoolExecutor(max_workers=max_workers)
        self._engine = _SyncEngine.open(expanded_path)

    # ------------------------------------------------------------------
    # Internal dispatch
    # ------------------------------------------------------------------

    async def _run(self, method: str, *args):
        """Look up *method* on the sync engine and invoke it in a thread."""
        # Validate against the real Engine class (not the instance, which
        # may be a MagicMock in test suites that auto-creates attributes).
        if not hasattr(_SYNC_ENGINE_CLASS, method):
            msg = f"Engine has no method named {method!r}"
            raise AttributeError(msg)
        fn = getattr(self._engine, method)
        # Build a truncated args summary without materialising the full repr
        # of large arguments (``str(args)[:200]`` would allocate a 100 KB+
        # string for large memory content).
        args_summary = _truncated_args_summary(args)
        start = time.monotonic()
        try:
            loop = asyncio.get_running_loop()
            result = await loop.run_in_executor(self._pool, fn, *args)
        except Exception:
            logger.exception("bridge_call_failed", method=method, args_summary=args_summary)
            raise
        duration_ms = round((time.monotonic() - start) * 1000, 1)
        logger.info(
            "bridge_call_end",
            method=method,
            args_summary=args_summary,
            duration_ms=duration_ms,
        )
        return result

    # ------------------------------------------------------------------
    # Session CRUD
    # ------------------------------------------------------------------

    async def create_session(self, session: dict) -> dict:
        result = await self._run("create_session", json.dumps(session))
        return json.loads(result)

    async def get_session(self, id: str) -> dict | None:
        result = await self._run("get_session", id)
        return json.loads(result) if result else None

    async def list_sessions(self, filter: dict | None = None, limit: int = 100, offset: int = 0) -> list[dict]:
        filter_dict = dict(filter) if filter is not None else {}
        filter_dict["limit"] = limit
        filter_dict["offset"] = offset
        filter_json = json.dumps(filter_dict)
        result = await self._run("list_sessions", filter_json)
        return json.loads(result)

    async def update_session(self, id: str, patch: dict) -> dict:
        result = await self._run("update_session", id, json.dumps(patch))
        return json.loads(result)

    async def delete_session(self, id: str) -> None:
        await self._run("delete_session", id)

    async def count_sessions(self, filter: dict | None = None) -> int:
        filter_json = json.dumps(filter) if filter is not None else "{}"
        return await self._run("count_sessions", filter_json)

    # ------------------------------------------------------------------
    # Memory CRUD
    # ------------------------------------------------------------------

    async def create_memory(self, memory: dict) -> dict:
        content = memory.get("content", "")
        if len(content.encode("utf-8")) >= _LARGE_CONTENT_THRESHOLD:
            meta = {k: v for k, v in memory.items() if k != "content"}
            result = await self._run(
                "create_memory_bytes",
                json.dumps(meta),
                content.encode("utf-8"),
            )
        else:
            result = await self._run("create_memory", json.dumps(memory))
        return json.loads(result)

    async def get_memory(self, id: str) -> dict | None:
        result = await self._run("get_memory", id)
        return json.loads(result) if result else None

    async def search_memories(self, query: dict, limit: int = 100, offset: int = 0) -> list[dict]:
        query_dict = dict(query)
        query_dict["limit"] = limit
        query_dict["offset"] = offset
        result = await self._run("search_memories", json.dumps(query_dict))
        return json.loads(result)

    async def update_memory(self, id: str, patch: dict) -> dict | None:
        content = patch.get("content", "")
        if len(content.encode("utf-8")) >= _LARGE_CONTENT_THRESHOLD:
            meta = {k: v for k, v in patch.items() if k != "content"}
            result = await self._run(
                "update_memory_bytes",
                id,
                json.dumps(meta),
                content.encode("utf-8"),
            )
        else:
            result = await self._run("update_memory", id, json.dumps(patch))
        return json.loads(result) if result else None

    async def delete_memory(self, id: str) -> None:
        await self._run("delete_memory", id)

    async def count_memories(self, query: dict) -> int:
        return await self._run("count_memories", json.dumps(query))

    # ------------------------------------------------------------------
    # Agent CRUD
    # ------------------------------------------------------------------

    async def create_agent(self, agent: dict) -> dict:
        result = await self._run("create_agent", json.dumps(agent))
        return json.loads(result)

    async def get_agent(self, id: str) -> dict | None:
        result = await self._run("get_agent", id)
        return json.loads(result) if result else None

    async def list_agents(self, filter: dict | None = None, limit: int = 100, offset: int = 0) -> list[dict]:
        filter_dict = dict(filter) if filter is not None else {}
        filter_dict["limit"] = limit
        filter_dict["offset"] = offset
        filter_json = json.dumps(filter_dict)
        result = await self._run("list_agents", filter_json)
        return json.loads(result)

    async def update_agent(self, id: str, patch: dict) -> dict:
        result = await self._run("update_agent", id, json.dumps(patch))
        return json.loads(result)

    async def delete_agent(self, id: str) -> None:
        await self._run("delete_agent", id)

    # ------------------------------------------------------------------
    # Skill CRUD
    # ------------------------------------------------------------------

    async def create_skill(self, skill: dict) -> dict:
        result = await self._run("create_skill", json.dumps(skill))
        return json.loads(result)

    async def get_skill(self, id: str) -> dict | None:
        result = await self._run("get_skill", id)
        return json.loads(result) if result else None

    async def list_skills(self, filter: dict | None = None, limit: int = 100, offset: int = 0) -> list[dict]:
        filter_dict = dict(filter) if filter is not None else {}
        filter_dict["limit"] = limit
        filter_dict["offset"] = offset
        filter_json = json.dumps(filter_dict)
        result = await self._run("list_skills", filter_json)
        return json.loads(result)

    async def update_skill(self, id: str, patch: dict) -> dict:
        result = await self._run("update_skill", id, json.dumps(patch))
        return json.loads(result)

    async def delete_skill(self, id: str) -> None:
        await self._run("delete_skill", id)

    # ------------------------------------------------------------------
    # Settings
    # ------------------------------------------------------------------

    async def set_setting(self, key: str, value: str) -> None:
        await self._run("set_setting", key, value)

    async def get_setting(self, key: str) -> str | None:
        return await self._run("get_setting", key)

    # ------------------------------------------------------------------
    # Audit
    # ------------------------------------------------------------------

    async def log_audit(self, entry: dict) -> None:
        await self._run("log_audit", json.dumps(entry))

    async def query_audit(self, filter: dict) -> list[dict]:
        result = await self._run("query_audit", json.dumps(filter))
        return json.loads(result)

    # ------------------------------------------------------------------
    # Maintenance
    # ------------------------------------------------------------------

    async def flush(self) -> None:
        await self._run("flush")

    async def checkpoint(self) -> int:
        return await self._run("checkpoint")

    async def storage_size(self) -> dict:
        result = await self._run("storage_size")
        return json.loads(result)

    async def status(self) -> dict:
        result = await self._run("status")
        return json.loads(result)

    async def clear_cache(self) -> None:
        await self._run("clear_cache")

    async def cache_telemetry(self) -> dict:
        result = await self._run("cache_telemetry")
        return json.loads(result)

    async def clear_cache_type(self, entity_type: str) -> None:
        await self._run("clear_cache_type", entity_type)
