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
import traceback
from concurrent.futures import ThreadPoolExecutor
from datetime import datetime, timezone
from pathlib import Path
from unittest.mock import Mock

import structlog

try:
    from contexter_core import Engine as _SyncEngine
except ImportError as exc:  # pragma: no cover - import guard
    raise ImportError(
        "contexter_core (the Rust PyO3 extension) is not installed. "
        "Install it from the contexter-core crate (maturin build + pip install) "
        "before running the server — the server refuses to run on mocks."
    ) from exc

# Capture the raw class before any test patching so _run can validate
# method existence independently of a potentially-mocked instance.
_SYNC_ENGINE_CLASS = _SyncEngine

_LARGE_CONTENT_THRESHOLD = 102_400  # 100 KB

logger = structlog.get_logger(__name__)


def _snake_to_camel(name: str) -> str:
    """Convert a snake_case identifier to camelCase (``agent_id`` → ``agentId``)."""
    head, *tail = name.split("_")
    return head + "".join(part.capitalize() for part in tail)


def _camelize_payload_keys(payload: dict) -> dict:
    """Translate top-level snake_case keys to the engine's camelCase contract.

    The Rust engine's serde structs rename their fields to camelCase
    (``agentId``, ``memoryType``, ``sessionId``, ``entityType`` ...) at the
    boundary. Nested values — e.g. ``metadata`` maps, which the engine stores
    as opaque ``serde_json::Value`` — pass through untouched.
    """
    return {
        (_snake_to_camel(key) if isinstance(key, str) else key): value
        for key, value in payload.items()
    }


# Documented cap for content-bearing string/bytes args in bridge logs
# (REQ-BH-001): at most 64 characters/bytes of any content are exposed, so
# full content and secrets never reach the log (REQ-BH-002/003).
_ARG_SUMMARY_CAP = 64


def _truncated_args_summary(args: tuple, max_len: int = 200) -> str:
    """Build a truncated repr of *args* without constructing the full repr of
    large string/bytes elements.

    Unlike ``str(args)[:max_len]``, which calls ``repr()`` on every element
    and builds the entire string before slicing, this function truncates
    individual string/bytes arguments to avoid allocating large intermediate
    strings.  For a tuple containing a 100 KB string, the 100 KB+ repr is
    never materialised.

    Content-bearing string/bytes arguments are capped at ``_ARG_SUMMARY_CAP``
    (64) characters/bytes each; content exactly at the cap is logged in full
    (no marker), anything beyond it is truncated with ``...``. Empty string
    and bytes arguments render as a ``<empty>`` placeholder.
    """
    if not args:
        return "()"

    pieces: list[str] = []
    for arg in args:
        if isinstance(arg, str):
            if not arg:
                pieces.append("'<empty>'")
            elif len(arg) <= _ARG_SUMMARY_CAP:
                pieces.append(repr(arg))
            else:
                # Only construct repr of a short prefix — never the full string.
                snippet = arg[:_ARG_SUMMARY_CAP - 3]
                pieces.append(repr(snippet)[:-1] + "...'")
        elif isinstance(arg, bytes):
            if not arg:
                pieces.append("b'<empty>'")
            elif len(arg) <= _ARG_SUMMARY_CAP:
                pieces.append(repr(arg))
            else:
                snippet = arg[:_ARG_SUMMARY_CAP - 3]
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


#: Default server-side diagnostics log (override with ``CONTEXTER_LOG_FILE``) —
#: the same launch log the MCP launcher uses for engine-open failures, so
#: operator diagnostics for launch and runtime failures live in one place.
_DEFAULT_DIAGNOSTICS_LOG = Path.home() / ".contexter" / "logs" / "mcp-launch.log"

#: Cap for the diagnostics-log path on the concise stderr line, keeping the
#: operator-visible line bounded (<512 chars) even for pathological env values.
_DIAGNOSTICS_LOG_SUMMARY_CAP = 100


def _resolve_diagnostics_log_path() -> Path:
    """Return the server-side diagnostics log path.

    ``CONTEXTER_LOG_FILE`` overrides the default ``~/.contexter/logs/``
    location — the same override the MCP launcher's launch log honours.
    """
    override = os.environ.get("CONTEXTER_LOG_FILE", "").strip()
    return Path(override) if override else _DEFAULT_DIAGNOSTICS_LOG


def _write_runtime_failure_diagnostics(
    method: str, exc: BaseException, args_summary: str
) -> Path | None:
    """Persist full raw diagnostics for an engine-call failure (best-effort).

    Appends a structured record plus the full traceback to the diagnostics
    log file (the MCP launch log).  Never raises: a failure to log must not
    mask the original engine exception.

    Returns the log path on success, ``None`` when the log could not be
    written.
    """
    log_path = _resolve_diagnostics_log_path()
    try:
        log_path.parent.mkdir(parents=True, exist_ok=True)
        timestamp = datetime.now(timezone.utc).isoformat()
        record = (
            f"timestamp={timestamp} event=bridge_call_failed "
            f"method={method!r} args_summary={args_summary!r} "
            f"exception={type(exc).__name__}\n"
        )
        record += "".join(traceback.format_exception(exc))
        with open(log_path, "a", encoding="utf-8") as log_file:
            log_file.write(record + "\n")
        return log_path
    except Exception:
        return None


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
            env_val = os.environ.get("CONTEXTER_BRIDGE_POOL_SIZE", "")
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
        class_method = getattr(_SYNC_ENGINE_CLASS, method, None)
        if class_method is None and not isinstance(_SYNC_ENGINE_CLASS, Mock):
            msg = f"Engine has no method named {method!r}"
            raise AttributeError(msg)
        # Defense in depth: the MagicMock stub bug shipped because mock
        # attributes leaked into the live dispatch path. A mock attribute on a
        # REAL engine class or instance (the stub-leak pattern) must never be
        # executed — raise loudly instead of letting a MagicMock result corrupt
        # the async services. Wholesale mocks — the entire engine being a
        # unittest.mock object — are explicit test doubles and are tolerated.
        if not isinstance(_SYNC_ENGINE_CLASS, Mock) and isinstance(class_method, Mock):
            raise TypeError(
                f"Engine method {method!r} resolves to a unittest.mock object "
                f"({type(class_method).__name__}) on the Engine class; refusing "
                "to dispatch mocked storage calls"
            )
        fn = getattr(self._engine, method)
        if not isinstance(self._engine, Mock) and isinstance(fn, Mock):
            raise TypeError(
                f"Engine method {method!r} resolves to a unittest.mock object "
                f"({type(fn).__name__}) on the engine instance; refusing to "
                "dispatch mocked storage calls"
            )
        # Build a truncated args summary without materialising the full repr
        # of large arguments (``str(args)[:200]`` would allocate a 100 KB+
        # string for large memory content).
        args_summary = _truncated_args_summary(args)
        start = time.monotonic()
        try:
            loop = asyncio.get_running_loop()
            result = await loop.run_in_executor(self._pool, fn, *args)
        except Exception as exc:
            # Engine-failure diagnostics: the full traceback goes to the
            # diagnostics log file (the MCP launch log), while stderr gets ONE
            # concise structured line — kind + bounded context, no exc_info —
            # so client-visible stderr stays bounded (<512 chars, no raw
            # traceback) and stdout remains pure (REQ-EFS-001..003).
            diagnostics_log = _write_runtime_failure_diagnostics(
                method, exc, args_summary
            )
            error_context = {
                "method": method,
                "args_summary": args_summary,
                # Not named ``exception``: structlog's ConsoleRenderer treats a
                # key literally called ``exception`` as a special trailing
                # line, which would leak the raw exception text to stderr.
                "exception_type": type(exc).__name__,
            }
            if diagnostics_log is not None:
                path_str = str(diagnostics_log)
                error_context["diagnostics_log"] = (
                    path_str
                    if len(path_str) <= _DIAGNOSTICS_LOG_SUMMARY_CAP
                    else path_str[: _DIAGNOSTICS_LOG_SUMMARY_CAP - 3] + "..."
                )
            logger.error("bridge_call_failed", **error_context)
            raise
        duration_ms = round((time.monotonic() - start) * 1000, 1)
        # Per-call event: DEBUG, not INFO — INFO is reserved for lifecycle
        # and error events (REQ-PLB-001 / PF-05).
        logger.debug(
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
        result = await self._run("create_session", json.dumps(_camelize_payload_keys(session)))
        return json.loads(result)

    async def get_session(self, id: str) -> dict | None:
        result = await self._run("get_session", id)
        return json.loads(result) if result else None

    async def list_sessions(self, filter: dict | None = None, limit: int = 100, offset: int = 0) -> list[dict]:
        filter_dict = dict(filter) if filter is not None else {}
        filter_dict["limit"] = limit
        filter_dict["offset"] = offset
        filter_json = json.dumps(_camelize_payload_keys(filter_dict))
        result = await self._run("list_sessions", filter_json)
        return json.loads(result)

    async def update_session(self, id: str, patch: dict) -> dict:
        result = await self._run("update_session", id, json.dumps(_camelize_payload_keys(patch)))
        return json.loads(result)

    async def delete_session(self, id: str) -> None:
        await self._run("delete_session", id)

    async def count_sessions(self, filter: dict | None = None) -> int:
        filter_json = json.dumps(_camelize_payload_keys(filter)) if filter is not None else "{}"
        return await self._run("count_sessions", filter_json)

    # ------------------------------------------------------------------
    # Memory CRUD
    # ------------------------------------------------------------------

    async def create_memory(self, memory: dict) -> dict:
        content = memory.get("content", "")
        # Encode once and reuse for both the size check and the payload
        # (REQ-BD-002): the bytes path must never encode content twice.
        content_bytes = content.encode("utf-8")
        if len(content_bytes) >= _LARGE_CONTENT_THRESHOLD:
            meta = {k: v for k, v in memory.items() if k != "content"}
            # The Rust NewMemory struct requires a `content` field in the meta
            # JSON; the raw bytes argument overwrites it, so an empty
            # placeholder satisfies serde without duplicating the full content
            # in the JSON payload.
            meta["content"] = ""
            result = await self._run(
                "create_memory_bytes",
                json.dumps(_camelize_payload_keys(meta)),
                content_bytes,
            )
        else:
            result = await self._run("create_memory", json.dumps(_camelize_payload_keys(memory)))
        return json.loads(result)

    async def get_memory(self, id: str) -> dict | None:
        result = await self._run("get_memory", id)
        return json.loads(result) if result else None

    async def search_memories(self, query: dict, limit: int = 100, offset: int = 0) -> list[dict]:
        query_dict = dict(query)
        query_dict["limit"] = limit
        query_dict["offset"] = offset
        result = await self._run("search_memories", json.dumps(_camelize_payload_keys(query_dict)))
        return json.loads(result)

    async def update_memory(self, id: str, patch: dict) -> dict | None:
        content = patch.get("content", "")
        # Encode once and reuse for both the size check and the payload
        # (REQ-BD-002): the bytes path must never encode content twice.
        content_bytes = content.encode("utf-8")
        if len(content_bytes) >= _LARGE_CONTENT_THRESHOLD:
            meta = {k: v for k, v in patch.items() if k != "content"}
            result = await self._run(
                "update_memory_bytes",
                id,
                json.dumps(_camelize_payload_keys(meta)),
                content_bytes,
            )
        else:
            result = await self._run("update_memory", id, json.dumps(_camelize_payload_keys(patch)))
        return json.loads(result) if result else None

    async def delete_memory(self, id: str) -> None:
        await self._run("delete_memory", id)

    async def count_memories(self, query: dict) -> int:
        return await self._run("count_memories", json.dumps(_camelize_payload_keys(query)))

    # ------------------------------------------------------------------
    # Agent CRUD
    # ------------------------------------------------------------------

    async def create_agent(self, agent: dict) -> dict:
        result = await self._run("create_agent", json.dumps(_camelize_payload_keys(agent)))
        return json.loads(result)

    async def get_agent(self, id: str) -> dict | None:
        result = await self._run("get_agent", id)
        return json.loads(result) if result else None

    async def list_agents(self, filter: dict | None = None, limit: int = 100, offset: int = 0) -> list[dict]:
        filter_dict = dict(filter) if filter is not None else {}
        filter_dict["limit"] = limit
        filter_dict["offset"] = offset
        filter_json = json.dumps(_camelize_payload_keys(filter_dict))
        result = await self._run("list_agents", filter_json)
        return json.loads(result)

    async def count_agents(self, filter: dict | None = None) -> int:
        filter_json = json.dumps(_camelize_payload_keys(filter)) if filter is not None else "{}"
        return await self._run("count_agents", filter_json)

    async def update_agent(self, id: str, patch: dict) -> dict:
        result = await self._run("update_agent", id, json.dumps(_camelize_payload_keys(patch)))
        return json.loads(result)

    async def delete_agent(self, id: str) -> None:
        await self._run("delete_agent", id)

    # ------------------------------------------------------------------
    # Skill CRUD
    # ------------------------------------------------------------------

    async def create_skill(self, skill: dict) -> dict:
        result = await self._run("create_skill", json.dumps(_camelize_payload_keys(skill)))
        return json.loads(result)

    async def get_skill(self, id: str) -> dict | None:
        result = await self._run("get_skill", id)
        return json.loads(result) if result else None

    async def list_skills(self, filter: dict | None = None, limit: int = 100, offset: int = 0) -> list[dict]:
        filter_dict = dict(filter) if filter is not None else {}
        filter_dict["limit"] = limit
        filter_dict["offset"] = offset
        filter_json = json.dumps(_camelize_payload_keys(filter_dict))
        result = await self._run("list_skills", filter_json)
        return json.loads(result)

    async def count_skills(self, filter: dict | None = None) -> int:
        filter_json = json.dumps(_camelize_payload_keys(filter)) if filter is not None else "{}"
        return await self._run("count_skills", filter_json)

    async def update_skill(self, id: str, patch: dict) -> dict:
        result = await self._run("update_skill", id, json.dumps(_camelize_payload_keys(patch)))
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
        await self._run("log_audit", json.dumps(_camelize_payload_keys(entry)))

    async def query_audit(self, filter: dict) -> list[dict]:
        result = await self._run("query_audit", json.dumps(_camelize_payload_keys(filter)))
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
