"""Pure async handler functions for MCP tools and resources.

Each handler accepts service instances via keyword arguments, making them
directly testable without the FastMCP framework.

Error contract (frozen): every failure path raises :class:`HandlerError`
— never a ``{"error": ...}`` success payload. Not-found entities use the
``Resource not found: <id>`` convention. Validation failures never echo
unbounded client input.

Observability: each handler emits structured logs (call received, auth
decision, engine result with duration, error) carrying a correlation id.
Per-request logs (call received, auth decision, engine result) are emitted
at DEBUG; INFO is reserved for lifecycle and error events, and error logs
use ERROR (REQ-PLB-001). Content payloads and secrets are never logged.
"""

import time
from typing import NoReturn
from uuid import UUID, uuid4

from structlog import get_logger

from contexter_server.mcp_tools.auth import require_api_key
from contexter_server.mcp_tools.errors import (
    DEFAULT_EXPORT_FORMAT,
    DEFAULT_SEARCH_LIMIT,
    EXPORT_FORMATS,
    MAX_CONTENT_LENGTH,
    MAX_QUERY_LENGTH,
    MAX_SEARCH_LIMIT,
    HandlerError,
    not_found_error,
    storage_error,
    validation_error,
)
from contexter_server.models.export import ExportRequest
from contexter_server.models.memory import MemoryCreate
from contexter_server.models.search import SearchQuery
from contexter_server.models.session import SessionFilter
from contexter_server.services.session_service import MAX_SESSION_LIST_LIMIT

logger = get_logger(__name__)


# ── Shared helpers ───────────────────────────────────────────────────────


def _clamp(value: int | None, default: int, maximum: int, minimum: int = 0) -> int:
    """Clamp ``value`` into ``[minimum, maximum]``, keeping ``default`` when None."""
    if value is None:
        return default
    return max(minimum, min(value, maximum))


def _clamp_session_list_limit(value: int | None) -> int | None:
    """Clamp the recent-sessions limit to the service rules (REQ-HLP-001).

    Negative and zero clamp to 0; values above ``MAX_SESSION_LIST_LIMIT``
    clamp to the documented maximum. ``None`` passes through unchanged so
    the service applies the engine default (100) (REQ-HLP-003).
    """
    if value is None:
        return None
    return max(0, min(value, MAX_SESSION_LIST_LIMIT))


def _bounded(value: object, max_chars: int = 64) -> str:
    """Truncate a value for safe inclusion in messages and logs (REQ-IV-005)."""
    text = str(value)
    if len(text) <= max_chars:
        return text
    return f"{text[: max_chars - 1]}…"


def _validate_content(content: str, log: object, started: float) -> None:
    """Reject empty/whitespace-only or oversized content (EC-006, REQ-IV-001/004)."""
    if not content or not content.strip():
        _raise_structured_error(log, validation_error("content must not be empty"), started)
    if len(content) > MAX_CONTENT_LENGTH:
        _raise_structured_error(
            log,
            validation_error(f"content exceeds maximum length of {MAX_CONTENT_LENGTH}"),
            started,
        )


def _validate_query(query: str, log: object, started: float) -> None:
    """Reject empty or oversized search query (REQ-IV-004, EC-IV-009)."""
    if not query or not query.strip():
        _raise_structured_error(log, validation_error("query must not be empty"), started)
    if len(query) > MAX_QUERY_LENGTH:
        _raise_structured_error(
            log,
            validation_error(f"query exceeds maximum length of {MAX_QUERY_LENGTH}"),
            started,
        )


def _validate_export_format(fmt: str, log: object, started: float) -> None:
    """Reject unsupported export formats (EC-012, REQ-IV-002)."""
    if fmt not in EXPORT_FORMATS:
        supported = ", ".join(sorted(EXPORT_FORMATS))
        _raise_structured_error(
            log,
            validation_error(
                f"unsupported export format: {_bounded(fmt)!r} (supported: {supported})"
            ),
            started,
        )


def _log_bind(tool: str) -> object:
    """Return a structlog logger bound with tool name and a fresh correlation id."""
    return logger.bind(correlation_id=uuid4().hex, tool=tool)


def _raise_structured_error(log: object, error: HandlerError, started: float) -> NoReturn:
    """Log a bounded structured error event, then raise the handler error.

    Only the machine-readable ``kind`` and duration are logged — never the
    message — so unbounded client input and payload content stay out of the
    logs (REQ-HO-001/002, REQ-IV-005).
    """
    duration_ms = round((time.monotonic() - started) * 1000, 3)
    log.error("handler_error", error_kind=error.kind, duration_ms=duration_ms)
    raise error


# ── Tool Handlers ───────────────────────────────────────────────────────


async def handle_store_memory(
    session_id: str,
    role: str,
    content: str,
    *,
    _api_key: str | None = None,
    memory_service=None,
    session_service=None,
) -> dict:
    """Store a new memory entry in an existing session.

    Looks up the session to derive the agent_id, then delegates to
    MemoryService.create().
    """
    log = _log_bind("store_memory")
    log.debug("call_received", session_id=_bounded(session_id))
    started = time.monotonic()

    require_api_key(_api_key)
    log.debug("auth_decision", allowed=True)

    if memory_service is None or session_service is None:
        _raise_structured_error(
            log, storage_error("MCP server not connected to storage"), started
        )

    _validate_content(content, log, started)

    try:
        parsed_session_id = UUID(session_id)
    except ValueError:
        _raise_structured_error(
            log, validation_error("invalid session_id (not a valid UUID)"), started
        )

    session = await session_service.get(session_id)
    if session is None:
        _raise_structured_error(log, not_found_error(_bounded(session_id)), started)

    data = MemoryCreate(
        session_id=parsed_session_id,
        agent_id=session.agent_id,
        role=role,
        content=content,
    )
    memory = await memory_service.create(data)
    duration_ms = (time.monotonic() - started) * 1000
    log.debug("engine_result", outcome="success", duration_ms=round(duration_ms, 3))
    return {
        "memory_id": str(memory.id),
        "created_at": memory.created_at.isoformat(),
    }


async def handle_search_memories(
    query: str,
    type: str | None = None,
    project: str | None = None,
    limit: int | None = None,
    *,
    _api_key: str | None = None,
    memory_service=None,
) -> dict:
    """Search memories across the system."""
    log = _log_bind("search_memories")
    log.debug("call_received")
    started = time.monotonic()

    require_api_key(_api_key)
    log.debug("auth_decision", allowed=True)

    if memory_service is None:
        _raise_structured_error(
            log, storage_error("MCP server not connected to storage"), started
        )

    _validate_query(query, log, started)
    clamped_limit = _clamp(limit, DEFAULT_SEARCH_LIMIT, MAX_SEARCH_LIMIT, minimum=1)

    search_query = SearchQuery(
        query=query,
        type=type,
        project=project,
        limit=clamped_limit,
    )
    response = await memory_service.search(search_query)
    duration_ms = (time.monotonic() - started) * 1000
    log.debug("engine_result", outcome="success", duration_ms=round(duration_ms, 3))
    return {
        "results": [
            {
                "id": str(r.id),
                "type": r.type,
                "score": r.score,
                "snippet": r.snippet,
            }
            for r in response.results
        ],
        "total": response.total,
    }


async def handle_get_session(
    id: str,
    *,
    _api_key: str | None = None,
    session_service=None,
) -> dict:
    """Get session details by ID."""
    log = _log_bind("get_session")
    log.debug("call_received", session_id=_bounded(id))
    started = time.monotonic()

    require_api_key(_api_key)
    log.debug("auth_decision", allowed=True)

    if session_service is None:
        _raise_structured_error(
            log, storage_error("MCP server not connected to storage"), started
        )

    session = await session_service.get(id)
    if session is None:
        _raise_structured_error(log, not_found_error(_bounded(id)), started)

    duration_ms = (time.monotonic() - started) * 1000
    log.debug("engine_result", outcome="success", duration_ms=round(duration_ms, 3))
    return {"session": session.model_dump(mode="json")}


async def handle_list_recent_sessions(
    limit: int | None = None,
    project: str | None = None,
    *,
    _api_key: str | None = None,
    session_service=None,
) -> dict:
    """List recent sessions, optionally filtered by project.

    The limit is clamped per the session list rules and pushed to the
    service, which forwards it to the engine (REQ-HLP-001). The service
    result is authoritative — no Python re-slice (REQ-HLP-002).
    """
    log = _log_bind("list_recent_sessions")
    log.debug("call_received", project=_bounded(project))
    started = time.monotonic()

    require_api_key(_api_key)
    log.debug("auth_decision", allowed=True)

    if session_service is None:
        _raise_structured_error(
            log, storage_error("MCP server not connected to storage"), started
        )

    filter_obj = SessionFilter(project=project) if project else None
    sessions = await session_service.list(
        filter=filter_obj, limit=_clamp_session_list_limit(limit)
    )

    duration_ms = (time.monotonic() - started) * 1000
    log.debug("engine_result", outcome="success", duration_ms=round(duration_ms, 3))
    return {"sessions": [s.model_dump(mode="json") for s in sessions]}


async def handle_get_agent_info(
    id: str,
    *,
    _api_key: str | None = None,
    agent_service=None,
) -> dict:
    """Get agent configuration by ID."""
    log = _log_bind("get_agent_info")
    log.debug("call_received", agent_id=_bounded(id))
    started = time.monotonic()

    require_api_key(_api_key)
    log.debug("auth_decision", allowed=True)

    if agent_service is None:
        _raise_structured_error(
            log, storage_error("MCP server not connected to storage"), started
        )

    agent = await agent_service.get(id)
    if agent is None:
        _raise_structured_error(log, not_found_error(_bounded(id)), started)

    duration_ms = (time.monotonic() - started) * 1000
    log.debug("engine_result", outcome="success", duration_ms=round(duration_ms, 3))
    return {"agent": agent.model_dump(mode="json")}


async def handle_list_skills(
    type: str | None = None,
    *,
    _api_key: str | None = None,
    skill_service=None,
) -> dict:
    """List available skills, optionally filtered by type."""
    log = _log_bind("list_skills")
    log.debug("call_received", type=_bounded(type))
    started = time.monotonic()

    require_api_key(_api_key)
    log.debug("auth_decision", allowed=True)

    if skill_service is None:
        _raise_structured_error(
            log, storage_error("MCP server not connected to storage"), started
        )

    filter_obj: dict | None = None
    if type is not None:
        filter_obj = {"type": type}

    skills = await skill_service.list(filter=filter_obj)
    duration_ms = (time.monotonic() - started) * 1000
    log.debug("engine_result", outcome="success", duration_ms=round(duration_ms, 3))
    return {"skills": [s.model_dump(mode="json") for s in skills]}


async def handle_get_system_health(
    *,
    _api_key: str | None = None,
    analytics_service=None,
) -> dict:
    """Get system health status and resource usage."""
    log = _log_bind("get_system_health")
    log.debug("call_received")
    started = time.monotonic()

    require_api_key(_api_key)
    log.debug("auth_decision", allowed=True)

    if analytics_service is None:
        _raise_structured_error(
            log, storage_error("MCP server not connected to storage"), started
        )

    health = await analytics_service.get_health()
    duration_ms = (time.monotonic() - started) * 1000
    log.debug("engine_result", outcome="success", duration_ms=round(duration_ms, 3))
    return {
        "status": health.status,
        "uptime": health.uptime_seconds,
        "memory_usage": health.memory_usage_mb,
        "storage_size": health.storage_size_bytes,
    }


async def handle_export_data(
    format: str | None = None,
    entities: list[str] | None = None,
    *,
    _api_key: str | None = None,
    export_service=None,
) -> dict:
    """Export data in the specified format."""
    log = _log_bind("export_data")
    log.debug("call_received")
    started = time.monotonic()

    require_api_key(_api_key)
    log.debug("auth_decision", allowed=True)

    if export_service is None:
        _raise_structured_error(
            log, storage_error("MCP server not connected to storage"), started
        )

    export_format = format or DEFAULT_EXPORT_FORMAT
    _validate_export_format(export_format, log, started)

    request = ExportRequest(
        format=export_format,
        entities=entities or [],
    )
    status = await export_service.submit(request)
    duration_ms = (time.monotonic() - started) * 1000
    log.debug("engine_result", outcome="success", duration_ms=round(duration_ms, 3))
    return {
        "export_id": str(status.id),
        "status": status.status,
    }


# ── Resource Handlers ───────────────────────────────────────────────────


async def handle_session_resource(
    id: str,
    *,
    _api_key: str | None = None,
    session_service=None,
) -> str:
    """Get session data as a read-only MCP resource."""
    log = _log_bind("session_resource")
    log.debug("call_received", session_id=_bounded(id))
    started = time.monotonic()

    require_api_key(_api_key)
    log.debug("auth_decision", allowed=True)

    if session_service is None:
        _raise_structured_error(
            log, storage_error("MCP server not connected to storage"), started
        )

    session = await session_service.get(id)
    if session is None:
        _raise_structured_error(log, not_found_error(_bounded(id)), started)

    duration_ms = (time.monotonic() - started) * 1000
    log.debug("engine_result", outcome="success", duration_ms=round(duration_ms, 3))
    return session.model_dump_json(indent=2)


async def handle_memory_resource(
    id: str,
    *,
    _api_key: str | None = None,
    memory_service=None,
) -> str:
    """Get memory data as a read-only MCP resource."""
    log = _log_bind("memory_resource")
    log.debug("call_received", memory_id=_bounded(id))
    started = time.monotonic()

    require_api_key(_api_key)
    log.debug("auth_decision", allowed=True)

    if memory_service is None:
        _raise_structured_error(
            log, storage_error("MCP server not connected to storage"), started
        )

    memory = await memory_service.get(id)
    if memory is None:
        _raise_structured_error(log, not_found_error(_bounded(id)), started)

    duration_ms = (time.monotonic() - started) * 1000
    log.debug("engine_result", outcome="success", duration_ms=round(duration_ms, 3))
    return memory.model_dump_json(indent=2)


async def handle_agent_resource(
    id: str,
    *,
    _api_key: str | None = None,
    agent_service=None,
) -> str:
    """Get agent data as a read-only MCP resource."""
    log = _log_bind("agent_resource")
    log.debug("call_received", agent_id=_bounded(id))
    started = time.monotonic()

    require_api_key(_api_key)
    log.debug("auth_decision", allowed=True)

    if agent_service is None:
        _raise_structured_error(
            log, storage_error("MCP server not connected to storage"), started
        )

    agent = await agent_service.get(id)
    if agent is None:
        _raise_structured_error(log, not_found_error(_bounded(id)), started)

    duration_ms = (time.monotonic() - started) * 1000
    log.debug("engine_result", outcome="success", duration_ms=round(duration_ms, 3))
    return agent.model_dump_json(indent=2)


async def handle_analytics_overview_resource(
    *,
    _api_key: str | None = None,
    analytics_service=None,
) -> str:
    """Get analytics overview as a read-only MCP resource."""
    log = _log_bind("analytics_overview_resource")
    log.debug("call_received")
    started = time.monotonic()

    require_api_key(_api_key)
    log.debug("auth_decision", allowed=True)

    if analytics_service is None:
        _raise_structured_error(
            log, storage_error("MCP server not connected to storage"), started
        )

    overview = await analytics_service.get_overview()
    duration_ms = (time.monotonic() - started) * 1000
    log.debug("engine_result", outcome="success", duration_ms=round(duration_ms, 3))
    return overview.model_dump_json(indent=2)
