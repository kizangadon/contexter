"""FastMCP application factory for the Contexter MCP server.

Registers 8 tools and 4 read-only resources that delegate to the
service layer. Tool and resource handler logic lives in
``mcp_tools.handlers`` for direct testability.
"""

import os
from typing import Any

from structlog import get_logger

from contexter_server.mcp_tools.handlers import (
    handle_agent_resource,
    handle_export_data,
    handle_get_agent_info,
    handle_get_session,
    handle_get_system_health,
    handle_list_recent_sessions,
    handle_list_skills,
    handle_memory_resource,
    handle_search_memories,
    handle_session_resource,
    handle_analytics_overview_resource,
    handle_store_memory,
)

logger = get_logger(__name__)


def create_mcp_server(
    memory_service: Any = None,
    session_service: Any = None,
    agent_service: Any = None,
    skill_service: Any = None,
    analytics_service: Any = None,
    export_service: Any = None,
) -> Any:
    """Create and return the Contexter FastMCP server.

    Accepts optional service instances for delegation. When services are
    provided the tools and resources operate against live storage; without
    them they return error messages. This lets consumers create a minimal
    server for schema introspection even when storage is unavailable.

    Parameters
    ----------
    memory_service : MemoryService or None
    session_service : SessionService or None
    agent_service : AgentService or None
    skill_service : SkillService or None
    analytics_service : AnalyticsService or None
    export_service : ExportService or None

    Returns
    -------
    FastMCP instance, or ``None`` when ``fastmcp`` is not installed.
    """
    try:
        from fastmcp import FastMCP
    except ImportError:
        logger.warning("fastmcp not installed; MCP server unavailable")
        return None

    # Log authentication status. Each tool and resource handler validates
    # the ``_api_key`` parameter against this value via
    # ``require_api_key()``.
    api_key = os.environ.get("CONtexTER_API_KEY", "")
    if api_key:
        logger.info("mcp_server.api_key_configured")
    else:
        logger.warning(
            "CONtexTER_API_KEY not set — MCP server has no API key auth"
        )

    mcp = FastMCP(
        "contexter",
        instructions="Contexter agent memory system MCP interface",
    )

    # ------------------------------------------------------------------
    # Tools (8)
    # ------------------------------------------------------------------

    @mcp.tool()
    async def store_memory(
        session_id: str,
        role: str,
        content: str,
        tokens: int | None = None,
        tokenizer: str | None = None,
        model: str | None = None,
        _api_key: str | None = None,
    ) -> dict:
        """Store a new memory entry in an existing session."""
        return await handle_store_memory(
            session_id=session_id,
            role=role,
            content=content,
            tokens=tokens,
            tokenizer=tokenizer,
            model=model,
            _api_key=_api_key,
            memory_service=memory_service,
            session_service=session_service,
        )

    @mcp.tool()
    async def search_memories(
        query: str,
        type: str | None = None,
        project: str | None = None,
        limit: int | None = None,
        _api_key: str | None = None,
    ) -> dict:
        """Search memories across the system."""
        return await handle_search_memories(
            query=query,
            type=type,
            project=project,
            limit=limit,
            _api_key=_api_key,
            memory_service=memory_service,
        )

    @mcp.tool()
    async def get_session(
        id: str,
        _api_key: str | None = None,
    ) -> dict:
        """Get session details by ID."""
        return await handle_get_session(
            id=id,
            _api_key=_api_key,
            session_service=session_service,
        )

    @mcp.tool()
    async def list_recent_sessions(
        limit: int | None = None,
        project: str | None = None,
        _api_key: str | None = None,
    ) -> dict:
        """List recent sessions, optionally filtered by project."""
        return await handle_list_recent_sessions(
            limit=limit,
            project=project,
            _api_key=_api_key,
            session_service=session_service,
        )

    @mcp.tool()
    async def get_agent_info(
        id: str,
        _api_key: str | None = None,
    ) -> dict:
        """Get agent configuration by ID."""
        return await handle_get_agent_info(
            id=id,
            _api_key=_api_key,
            agent_service=agent_service,
        )

    @mcp.tool()
    async def list_skills(
        type: str | None = None,
        _api_key: str | None = None,
    ) -> dict:
        """List available skills, optionally filtered by type."""
        return await handle_list_skills(
            type=type,
            _api_key=_api_key,
            skill_service=skill_service,
        )

    @mcp.tool()
    async def get_system_health(
        _api_key: str | None = None,
    ) -> dict:
        """Get system health status and resource usage."""
        return await handle_get_system_health(
            _api_key=_api_key,
            analytics_service=analytics_service,
        )

    @mcp.tool()
    async def export_data(
        format: str | None = None,
        entities: list[str] | None = None,
        _api_key: str | None = None,
    ) -> dict:
        """Export data in the specified format."""
        return await handle_export_data(
            format=format,
            entities=entities,
            _api_key=_api_key,
            export_service=export_service,
        )

    # ------------------------------------------------------------------
    # Read-only resources (4)
    # ------------------------------------------------------------------

    @mcp.resource("contexter://session/{id}")
    async def session_resource(
        id: str,
        _api_key: str | None = None,
    ) -> str:
        """Return session data as a read-only resource."""
        return await handle_session_resource(
            id=id,
            _api_key=_api_key,
            session_service=session_service,
        )

    @mcp.resource("contexter://memory/{id}")
    async def memory_resource(
        id: str,
        _api_key: str | None = None,
    ) -> str:
        """Return memory data as a read-only resource."""
        return await handle_memory_resource(
            id=id,
            _api_key=_api_key,
            memory_service=memory_service,
        )

    @mcp.resource("contexter://agent/{id}")
    async def agent_resource(
        id: str,
        _api_key: str | None = None,
    ) -> str:
        """Return agent data as a read-only resource."""
        return await handle_agent_resource(
            id=id,
            _api_key=_api_key,
            agent_service=agent_service,
        )

    @mcp.resource("contexter://analytics/overview{?_api_key}")
    async def analytics_overview_resource(
        _api_key: str | None = None,
    ) -> str:
        """Return analytics overview as a read-only resource."""
        return await handle_analytics_overview_resource(
            _api_key=_api_key,
            analytics_service=analytics_service,
        )

    return mcp


mcp = create_mcp_server()
