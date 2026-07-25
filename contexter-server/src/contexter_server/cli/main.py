"""Click-based CLI entry point for Contexter admin and diagnostics.

Usage:
    contexter --help
    contexter session create --agent-id <uuid> --project <name> [--name <name>]
    contexter session list [--project <name>] [--status <status>]
    contexter session get <session_id>
    contexter session delete <session_id>
    contexter memory create --session-id <uuid> --agent-id <uuid> --role <role> --content <text>
    contexter memory search <query> [--type <type>] [--project <project>] [--limit <n>]
    contexter status
    contexter export [--format <fmt>] [--entities <entities>]
    contexter gc
"""

from pathlib import Path

import click

from contexter_server.cli.export_commands import export
from contexter_server.cli.memory_commands import memory
from contexter_server.cli.session_commands import session
from contexter_server.cli.status_commands import gc_cmd
from contexter_server.cli.status_commands import status as status_cmd


@click.group()
@click.option(
    "--engine-path",
    envvar="CONTEXTER_PATH",
    default=str(Path.home() / ".contexter"),
    show_default=True,
    help="Path to the Contexter storage engine directory.",
)
@click.pass_context
def cli(ctx: click.Context, engine_path: str) -> None:
    """Contexter — agent memory management system.

    Admin and diagnostics tool for managing sessions, memories, exports,
    and system status of the Contexter storage layer.
    """
    ctx.ensure_object(dict)
    ctx.obj["engine_path"] = engine_path


# Register command groups
cli.add_command(session)
cli.add_command(memory)
cli.add_command(status_cmd)
cli.add_command(export)
cli.add_command(gc_cmd)
