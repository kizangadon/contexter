"""Status and GC CLI commands — system health and maintenance."""

import asyncio

import click
from structlog import get_logger

from contexter_server.core.bridge import StorageEngine
from contexter_server.services.analytics_service import AnalyticsService

logger = get_logger(__name__)


# ---------------------------------------------------------------------------
# contexter status
# ---------------------------------------------------------------------------


@click.command(name="status")
@click.pass_context
def status(ctx: click.Context) -> None:
    """Show system status, health, and analytics overview."""
    engine = StorageEngine(ctx.obj["engine_path"])
    service = AnalyticsService(engine)

    try:
        overview, health, performance, resources, version = asyncio.run(
            _fetch_status(service, engine)
        )
    except Exception as e:
        logger.exception("status.fetch_failed")
        raise click.ClickException("Failed to fetch system status. Check logs for details.") from e

    click.echo("Contexter System Status")
    click.echo("═" * 60)
    click.echo(f"  Health:               {health.status}")
    click.echo(f"  Version:              {version}")
    click.echo(f"  Uptime:               {_format_uptime(health.uptime_seconds)}")
    click.echo(f"  Memory usage:         {health.memory_usage_mb:.1f} MB")
    click.echo(f"  Storage size:         {_format_bytes(health.storage_size_bytes)}")
    click.echo(f"  Cache entries:        {health.cache_entries}")
    click.echo()
    click.echo(f"  Sessions:             {overview.total_sessions}")
    click.echo(f"  Memories:             {overview.total_memories}")
    click.echo(f"  Agents:               {overview.total_agents}")
    click.echo(f"  Skills:               {overview.total_skills}")
    click.echo()
    click.echo(f"  Avg response time:    {performance.avg_response_time_ms:.1f} ms")
    click.echo(f"  Total operations:     {performance.total_operations}")
    click.echo(f"  Cache hit rate:       {performance.cache_hit_rate:.1%}")
    click.echo()
    click.echo(f"  CPU:                  {resources.cpu_percent:.1f}%")
    click.echo(f"  Storage (MB):         {resources.storage_mb:.1f} MB")


async def _fetch_status(service: AnalyticsService, engine: StorageEngine) -> tuple:
    """Fetch all status information concurrently."""
    overview = await service.get_overview()
    health = await service.get_health()
    performance = await service.get_performance()
    resources = await service.get_resources()
    version = await _read_engine_version(engine)
    return overview, health, performance, resources, version


async def _read_engine_version(engine: StorageEngine) -> str:
    """Read the engine version from ``status()`` for display.

    The analytics domain models do not carry the engine version, so the
    CLI reads it from the bridge directly. A missing or non-string
    ``version`` degrades to ``"unknown"`` rather than crashing the report;
    an engine failure propagates to the command's error path unchanged.
    """
    raw = await engine.status()
    if not isinstance(raw, dict):
        logger.warning(
            "status.version_payload_invalid",
            payload_type=type(raw).__name__,
        )
        return "unknown"
    version = raw.get("version")
    if not isinstance(version, str) or not version:
        logger.warning("status.version_missing")
        return "unknown"
    return version


def _format_uptime(seconds: int) -> str:
    """Format uptime seconds into a human-readable string."""
    days, remainder = divmod(seconds, 86400)
    hours, remainder = divmod(remainder, 3600)
    minutes, secs = divmod(remainder, 60)

    parts = []
    if days > 0:
        parts.append(f"{days}d")
    if hours > 0:
        parts.append(f"{hours}h")
    if minutes > 0:
        parts.append(f"{minutes}m")
    parts.append(f"{secs}s")
    return " ".join(parts)


def _format_bytes(num_bytes: int) -> str:
    """Format bytes into a human-readable size string."""
    for unit in ("B", "KB", "MB", "GB"):
        if num_bytes < 1024:
            return f"{num_bytes:.1f} {unit}"
        num_bytes /= 1024
    return f"{num_bytes:.1f} TB"


# ---------------------------------------------------------------------------
# contexter gc
# ---------------------------------------------------------------------------


@click.command(name="gc")
@click.pass_context
def gc_cmd(ctx: click.Context) -> None:
    """Run garbage collection — flush WAL and create a checkpoint."""
    engine = StorageEngine(ctx.obj["engine_path"])
    try:

        async def _run_gc() -> int:
            await engine.flush()
            checkpoint_id = await engine.checkpoint()
            return checkpoint_id

        checkpoint_id = asyncio.run(_run_gc())
        click.echo(f"Garbage collection complete. Checkpoint ID: {checkpoint_id}")
    except Exception as e:
        logger.exception("gc.failed")
        raise click.ClickException("Garbage collection failed. Check logs for details.") from e
