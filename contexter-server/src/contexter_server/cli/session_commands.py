"""Session CLI commands — CRUD operations for Session aggregates."""

import asyncio
import json
from datetime import datetime
from typing import Any

import click

from contexter_server.core.bridge import StorageEngine
from contexter_server.models.session import SessionCreate, SessionFilter
from contexter_server.services.session_service import SessionService


def _format_session(s: Any) -> dict[str, Any]:
    """Serialize a Session model to a JSON-safe dict for display."""
    return {
        "id": str(s.id),
        "agent_id": str(s.agent_id),
        "project": s.project or "",
        "name": s.name or "",
        "status": s.status,
        "started_at": s.started_at.isoformat() if s.started_at else None,
        "updated_at": s.updated_at.isoformat() if s.updated_at else None,
        "completed_at": s.completed_at.isoformat() if s.completed_at else None,
    }


def _print_session(s: Any) -> None:
    """Print a formatted session."""
    data = _format_session(s)
    click.echo(f"  ID:          {data['id']}")
    click.echo(f"  Agent ID:    {data['agent_id']}")
    click.echo(f"  Project:     {data['project']}")
    click.echo(f"  Name:        {data['name'] or ''}")
    click.echo(f"  Status:      {data['status']}")
    click.echo(f"  Started:     {data['started_at']}")
    click.echo(f"  Updated:     {data['updated_at']}")
    if data["completed_at"]:
        click.echo(f"  Completed:   {data['completed_at']}")


def _get_service(engine_path: str) -> SessionService:
    """Build a SessionService from an engine path."""
    engine = StorageEngine(engine_path)
    return SessionService(engine)


@click.group(name="session")
def session() -> None:
    """Manage sessions — create, list, get, delete."""


@session.command(name="create")
@click.option("--agent-id", required=True, help="Agent UUID for the session")
@click.option("--project", required=True, help="Project name")
@click.option("--name", default=None, help="Optional session name")
@click.pass_context
def create_session(ctx: click.Context, agent_id: str, project: str, name: str | None) -> None:
    """Create a new session."""
    service = _get_service(ctx.obj["engine_path"])
    data = SessionCreate(
        agent_id=agent_id,
        project=project,
        name=name,
    )
    try:
        result = asyncio.run(service.create(data))
    except Exception as e:
        raise click.ClickException(str(e)) from e

    click.echo("Session created:")
    _print_session(result)


@session.command(name="list")
@click.option("--project", default=None, help="Filter by project name")
@click.option("--status", default=None, help="Filter by status (active, paused, completed, archived)")
@click.option("--json", "json_output", is_flag=True, help="Output as JSON")
@click.pass_context
def list_sessions(
    ctx: click.Context,
    project: str | None,
    status: str | None,
    json_output: bool,
) -> None:
    """List sessions with optional filters."""
    service = _get_service(ctx.obj["engine_path"])
    filter_obj = SessionFilter(project=project, status=status) if project or status else None
    try:
        results = asyncio.run(service.list(filter_obj))
    except Exception as e:
        raise click.ClickException(str(e)) from e

    if not results:
        click.echo("No sessions found.")
        return

    if json_output:
        click.echo(json.dumps([_format_session(s) for s in results], indent=2))
        return

    click.echo(f"Sessions ({len(results)}):")
    for i, s in enumerate(results, 1):
        click.echo(f"\n[{i}]")
        _print_session(s)


@session.command(name="get")
@click.argument("session_id")
@click.pass_context
def get_session(ctx: click.Context, session_id: str) -> None:
    """Get a session by ID."""
    service = _get_service(ctx.obj["engine_path"])
    try:
        result = asyncio.run(service.get(session_id))
    except Exception as e:
        raise click.ClickException(str(e)) from e

    if result is None:
        raise click.ClickException(f"Session not found: {session_id}")

    click.echo("Session:")
    _print_session(result)


@session.command(name="delete")
@click.argument("session_id")
@click.pass_context
def delete_session(ctx: click.Context, session_id: str) -> None:
    """Delete a session by ID."""
    service = _get_service(ctx.obj["engine_path"])
    try:
        asyncio.run(service.delete(session_id))
    except Exception as e:
        raise click.ClickException(str(e)) from e

    click.echo(f"Session deleted: {session_id}")
