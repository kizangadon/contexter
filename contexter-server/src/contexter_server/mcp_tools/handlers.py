"""Pure async handler functions for MCP tools and resources.

Each handler accepts service instances via keyword arguments, making them
directly testable without the FastMCP framework.
"""

from uuid import UUID
from typing import Any

from contexter_server.mcp_tools.auth import require_api_key
from contexter_server.models.export import ExportRequest
from contexter_server.models.memory import MemoryCreate
from contexter_server.models.search import SearchQuery
from contexter_server.models.session import SessionFilter


# ── Tool Handlers ───────────────────────────────────────────────────────


async def handle_store_memory(
    session_id: str,
    role: str,
    content: str,
    tokens: int | None = None,
    tokenizer: str | None = None,
    model: str | None = None,
    *,
    _api_key: str | None = None,
    memory_service: Any = None,
    session_service: Any = None,
) -> dict:
    """Store a new memory entry in an existing session.

    Looks up the session to derive the agent_id, then delegates to
    MemoryService.create().
    """
    require_api_key(_api_key)
    if memory_service is None or session_service is None:
        return {"error": "MCP server not connected to storage"}

    try:
        parsed_session_id = UUID(session_id)
    except ValueError:
        return {"error": f"invalid session_id (not a valid UUID): {session_id}"}

    session = await session_service.get(session_id)
    if session is None:
        return {"error": f"session not found: {session_id}"}

    data = MemoryCreate(
        session_id=parsed_session_id,
        agent_id=session.agent_id,
        role=role,
        content=content,
        tokens=tokens,
        tokenizer=tokenizer,
        model=model,
    )
    memory = await memory_service.create(data)
    return {
        "memory_id": str(memory.id),
        "created_at": memory.created_at.isoformat(),
    }


async def handle_search_memories(
    query: str,
    type_filter: str | None = None,
    project: str | None = None,
    limit: int | None = None,
    *,
    _api_key: str | None = None,
    memory_service: Any = None,
) -> dict:
    """Search memories across the system."""
    require_api_key(_api_key)
    if memory_service is None:
        return {"error": "MCP server not connected to storage"}

    search_query = SearchQuery(
        query=query,
        type=type_filter,
        project=project,
        limit=limit or 20,
    )
    response = await memory_service.search(search_query)
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
    session_service: Any = None,
) -> dict:
    """Get session details by ID."""
    require_api_key(_api_key)
    if session_service is None:
        return {"error": "MCP server not connected to storage"}

    session = await session_service.get(id)
    if session is None:
        return {"error": "not found"}

    return {"session": session.model_dump(mode="json")}


async def handle_list_recent_sessions(
    limit: int | None = None,
    project: str | None = None,
    *,
    _api_key: str | None = None,
    session_service: Any = None,
) -> dict:
    """List recent sessions, optionally filtered by project."""
    require_api_key(_api_key)
    if session_service is None:
        return {"error": "MCP server not connected to storage"}

    filter_obj = SessionFilter(project=project) if project else None
    sessions = await session_service.list(filter=filter_obj)

    if limit is not None:
        sessions = sessions[:limit]

    return {"sessions": [s.model_dump(mode="json") for s in sessions]}


async def handle_get_agent_info(
    id: str,
    *,
    _api_key: str | None = None,
    agent_service: Any = None,
) -> dict:
    """Get agent configuration by ID."""
    require_api_key(_api_key)
    if agent_service is None:
        return {"error": "MCP server not connected to storage"}

    agent = await agent_service.get(id)
    if agent is None:
        return {"error": "not found"}

    return {"agent": agent.model_dump(mode="json")}


async def handle_list_skills(
    type_filter: str | None = None,
    *,
    _api_key: str | None = None,
    skill_service: Any = None,
) -> dict:
    """List available skills, optionally filtered by type."""
    require_api_key(_api_key)
    if skill_service is None:
        return {"error": "MCP server not connected to storage"}

    filter_obj: dict | None = None
    if type_filter is not None:
        filter_obj = {"type": type_filter}

    skills = await skill_service.list(filter=filter_obj)
    return {"skills": [s.model_dump(mode="json") for s in skills]}


async def handle_get_system_health(
    *,
    _api_key: str | None = None,
    analytics_service: Any = None,
) -> dict:
    """Get system health status and resource usage."""
    require_api_key(_api_key)
    if analytics_service is None:
        return {"error": "MCP server not connected to storage"}

    health = await analytics_service.get_health()
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
    export_service: Any = None,
) -> dict:
    """Export data in the specified format."""
    require_api_key(_api_key)
    if export_service is None:
        return {"error": "MCP server not connected to storage"}

    request = ExportRequest(
        format=format or "json",
        entities=entities or [],
    )
    status = await export_service.submit(request)
    return {
        "export_id": str(status.id),
        "status": status.status,
    }


# ── Resource Handlers ───────────────────────────────────────────────────


async def handle_session_resource(
    id: str,
    *,
    _api_key: str | None = None,
    session_service: Any = None,
) -> str:
    """Get session data as a read-only MCP resource."""
    require_api_key(_api_key)
    if session_service is None:
        return "MCP server not connected to storage"

    session = await session_service.get(id)
    if session is None:
        return "Session not found"

    return session.model_dump_json(indent=2)


async def handle_memory_resource(
    id: str,
    *,
    _api_key: str | None = None,
    memory_service: Any = None,
) -> str:
    """Get memory data as a read-only MCP resource."""
    require_api_key(_api_key)
    if memory_service is None:
        return "MCP server not connected to storage"

    memory = await memory_service.get(id)
    if memory is None:
        return "Memory not found"

    return memory.model_dump_json(indent=2)


async def handle_agent_resource(
    id: str,
    *,
    _api_key: str | None = None,
    agent_service: Any = None,
) -> str:
    """Get agent data as a read-only MCP resource."""
    require_api_key(_api_key)
    if agent_service is None:
        return "MCP server not connected to storage"

    agent = await agent_service.get(id)
    if agent is None:
        return "Agent not found"

    return agent.model_dump_json(indent=2)


async def handle_analytics_overview_resource(
    *,
    _api_key: str | None = None,
    analytics_service: Any = None,
) -> str:
    """Get analytics overview as a read-only MCP resource."""
    require_api_key(_api_key)
    if analytics_service is None:
        return "MCP server not connected to storage"

    overview = await analytics_service.get_overview()
    return overview.model_dump_json(indent=2)
