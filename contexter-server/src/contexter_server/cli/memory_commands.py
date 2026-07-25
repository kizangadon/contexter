"""Memory CLI commands — create and search for Memory aggregates."""

import asyncio
import json
from typing import Any

import click

from contexter_server.core.bridge import StorageEngine
from contexter_server.models.memory import MemoryCreate
from contexter_server.models.search import SearchQuery
from contexter_server.services.memory_service import MemoryService


def _format_memory(m: Any) -> dict[str, Any]:
    """Serialize a Memory model to a JSON-safe dict for display."""
    return {
        "id": str(m.id),
        "session_id": str(m.session_id),
        "agent_id": str(m.agent_id),
        "role": m.role,
        "content": m.content[:200] + "..." if len(m.content) > 200 else m.content,
        "tokens": m.tokens,
        "created_at": m.created_at.isoformat() if m.created_at else None,
    }


def _format_search_result(r: Any) -> dict[str, Any]:
    """Serialize a SearchResult to a JSON-safe dict."""
    return {
        "id": str(r.id),
        "type": r.type,
        "score": r.score,
        "snippet": r.snippet,
        "data": {k: str(v) if not isinstance(v, (str, int, float, bool, list, dict)) else v for k, v in r.data.items()},
    }


def _print_memory(m: Any) -> None:
    """Print a formatted memory entry."""
    data = _format_memory(m)
    click.echo(f"  ID:          {data['id']}")
    click.echo(f"  Session ID:  {data['session_id']}")
    click.echo(f"  Agent ID:    {data['agent_id']}")
    click.echo(f"  Role:        {data['role']}")
    click.echo(f"  Content:     {data['content']}")
    if data["tokens"]:
        click.echo(f"  Tokens:      {data['tokens']}")
    click.echo(f"  Created:     {data['created_at']}")


def _print_search_result(r: Any) -> None:
    """Print a formatted search result."""
    data = _format_search_result(r)
    click.echo(f"  ID:          {data['id']}")
    click.echo(f"  Type:        {data['type']}")
    click.echo(f"  Score:       {data['score']:.4f}")
    if data["snippet"]:
        click.echo(f"  Snippet:     {data['snippet']}")


def _get_service(engine_path: str) -> MemoryService:
    """Build a MemoryService from an engine path."""
    engine = StorageEngine(engine_path)
    return MemoryService(engine)


@click.group(name="memory")
def memory() -> None:
    """Manage memories — create and search."""


@memory.command(name="create")
@click.option("--session-id", required=True, help="Session UUID")
@click.option("--agent-id", required=True, help="Agent UUID")
@click.option("--role", required=True, type=click.Choice(["user", "assistant", "system", "tool"]), help="Message role")
@click.option("--content", required=True, help="Memory content text")
@click.option("--tokens", type=int, default=None, help="Token count")
@click.pass_context
def create_memory(
    ctx: click.Context,
    session_id: str,
    agent_id: str,
    role: str,
    content: str,
    tokens: int | None,
) -> None:
    """Create a new memory entry."""
    service = _get_service(ctx.obj["engine_path"])
    data = MemoryCreate(
        session_id=session_id,
        agent_id=agent_id,
        role=role,
        content=content,
        tokens=tokens,
    )
    try:
        result = asyncio.run(service.create(data))
    except Exception as e:
        raise click.ClickException(str(e)) from e

    click.echo("Memory created:")
    _print_memory(result)


@memory.command(name="search")
@click.argument("query")
@click.option("--type", "memory_type", default=None, help="Filter by memory type")
@click.option("--project", default=None, help="Filter by project")
@click.option("--limit", type=int, default=20, help="Max results (1-100)")
@click.option("--json", "json_output", is_flag=True, help="Output as JSON")
@click.pass_context
def search_memories(
    ctx: click.Context,
    query: str,
    memory_type: str | None,
    project: str | None,
    limit: int,
    json_output: bool,
) -> None:
    """Search memories by query text."""
    service = _get_service(ctx.obj["engine_path"])
    search_query = SearchQuery(
        query=query,
        type=memory_type,
        project=project,
        limit=min(limit, 100),
    )
    try:
        response = asyncio.run(service.search(search_query))
    except Exception as e:
        raise click.ClickException(str(e)) from e

    if not response.results:
        click.echo("No results found.")
        return

    if json_output:
        click.echo(
            json.dumps(
                {
                    "total": response.total,
                    "page": response.page,
                    "limit": response.limit,
                    "results": [_format_search_result(r) for r in response.results],
                },
                indent=2,
            )
        )
        return

    click.echo(f"Results ({response.total} total, page {response.page}):")
    for i, r in enumerate(response.results, 1):
        click.echo(f"\n[{i}] (score: {r.score:.4f})")
        _print_search_result(r)
