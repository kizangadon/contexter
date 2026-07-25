"""Export CLI commands — submit and manage data exports."""

import asyncio
import json
from typing import Any

import click

from contexter_server.core.bridge import StorageEngine
from contexter_server.models.export import ExportRequest
from contexter_server.services.export_service import ExportService


def _format_export_status(s: Any) -> dict[str, Any]:
    """Serialize an ExportStatus model to a JSON-safe dict."""
    return {
        "id": str(s.id),
        "status": s.status,
        "progress": s.progress,
        "format": s.format,
        "created_at": s.created_at.isoformat() if s.created_at else None,
        "completed_at": s.completed_at.isoformat() if s.completed_at else None,
        "error": s.error,
    }


def _print_export_status(s: Any) -> None:
    """Print a formatted export status."""
    data = _format_export_status(s)
    click.echo(f"  ID:           {data['id']}")
    click.echo(f"  Status:       {data['status']}")
    click.echo(f"  Progress:     {data['progress']:.0%}")
    click.echo(f"  Format:       {data['format']}")
    click.echo(f"  Created:      {data['created_at']}")
    if data["completed_at"]:
        click.echo(f"  Completed:    {data['completed_at']}")
    if data["error"]:
        click.echo(f"  Error:        {data['error']}")


def _get_service(engine_path: str) -> ExportService:
    """Build an ExportService from an engine path."""
    engine = StorageEngine(engine_path)
    return ExportService(engine)


@click.command(name="export")
@click.option(
    "--format", "export_format",
    type=click.Choice(["json", "yaml", "csv"]),
    default="json",
    show_default=True,
    help="Export format",
)
@click.option(
    "--entities",
    default=None,
    help="Comma-separated entity types (sessions, memories, agents, skills)",
)
@click.option("--json", "json_output", is_flag=True, help="Output as JSON")
@click.pass_context
def export(
    ctx: click.Context,
    export_format: str,
    entities: str | None,
    json_output: bool,
) -> None:
    """Export data from the system.

    Exports entities in the requested format. Supports sessions, memories,
    agents, and skills. If no entities are specified, all are exported.
    """
    service = _get_service(ctx.obj["engine_path"])

    entity_list: list[str] = []
    if entities:
        entity_list = [e.strip() for e in entities.split(",") if e.strip()]

    request = ExportRequest(
        format=export_format,
        entities=entity_list,
    )

    try:
        status_result = asyncio.run(service.submit(request))
    except Exception as e:
        raise click.ClickException(str(e)) from e

    if json_output:
        click.echo(json.dumps(_format_export_status(status_result), indent=2))
        return

    click.echo("Export submitted:")
    _print_export_status(status_result)
