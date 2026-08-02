#!/usr/bin/env python3
"""Launcher for the Contexter MCP server (stdio transport).

Spawns the FastMCP server with all services wired to the storage engine.
OpenCode's MCP config runs this script as a subprocess.

Usage:
    python3 run_mcp.py          # Stdio transport (for OpenCode MCP)
    python3 run_mcp.py --sse    # SSE transport on port 8052

Failure behavior (defined, documented, tested):
    If the storage engine cannot be opened (RocksDB LOCK error, corrupt
    engine data, unwritable data dir), the launcher prints ONE clean
    structured error line to stderr — never a raw Python/Rust traceback —
    appends the full diagnostics (structured event + traceback) to the
    launch log, and exits with code ``ENGINE_OPEN_EXIT_CODE`` (2).
    Exit code 1 is reserved for a missing fastmcp installation.
"""

import os
import sys
import traceback
from datetime import datetime, timezone
from pathlib import Path
from typing import NoReturn

# Ensure contexter-server is on the path
sys.path.insert(0, str(Path(__file__).resolve().parent / "src"))

from contexter_server.core.bridge import StorageEngine
from contexter_server.mcp_server import create_mcp_server
from contexter_server.services import (
    MemoryService,
    SessionService,
    AgentService,
    SkillService,
    AnalyticsService,
    ExportService,
)

#: Exit code for engine-open failure (fastmcp-missing uses 1).
ENGINE_OPEN_EXIT_CODE = 2

#: Default server-side launch log (override with ``CONTEXTER_LOG_FILE``).
DEFAULT_LAUNCH_LOG = Path.home() / ".contexter" / "logs" / "mcp-launch.log"


def _resolve_launch_log_path() -> Path:
    """Return the server-side launch log path.

    ``CONTEXTER_LOG_FILE`` overrides the default ``~/.contexter/logs/``
    location so operators can pin the log independently of the engine path.
    """
    override = os.environ.get("CONTEXTER_LOG_FILE", "").strip()
    return Path(override) if override else DEFAULT_LAUNCH_LOG


def _write_launch_failure_log(engine_path: str, exc: BaseException) -> Path | None:
    """Persist full raw diagnostics for an engine-open failure (best-effort).

    Appends a structured record plus the full traceback to the launch log.
    Never raises: a failure to log must not mask the clean client error.

    Returns the log path on success, ``None`` when the log could not be
    written.
    """
    log_path = _resolve_launch_log_path()
    try:
        log_path.parent.mkdir(parents=True, exist_ok=True)
        timestamp = datetime.now(timezone.utc).isoformat()
        record = (
            f"timestamp={timestamp} event=engine_open_failed "
            f"engine_path={engine_path!r} exception={type(exc).__name__}\n"
        )
        record += "".join(traceback.format_exception(exc))
        with open(log_path, "a", encoding="utf-8") as log_file:
            log_file.write(record + "\n")
        return log_path
    except Exception:
        return None


def _fail_engine_open(engine_path: str, exc: BaseException) -> NoReturn:
    """Handle an engine-open failure at the launcher boundary.

    Persists full diagnostics server-side, prints ONE clean structured error
    line to stderr (no traceback — stderr is client-visible for an MCP
    subprocess), and exits with the documented nonzero code.
    """
    log_path = _write_launch_failure_log(engine_path, exc)
    reason = str(exc).splitlines()[0] if str(exc) else type(exc).__name__
    message = (
        f"contexter: engine_open_failed: could not open storage engine at "
        f"{engine_path!r}: {reason}"
    )
    if log_path is not None:
        message += f" (full diagnostics: {log_path})"
    print(message, file=sys.stderr)
    sys.exit(ENGINE_OPEN_EXIT_CODE)


def build_services(engine_path: str) -> dict:
    """Construct the six MCP services on the StorageEngine bridge.

    Mirrors ``contexter_server.main._create_services``: the MCP server talks
    to the real Rust engine through the same async bridge as the REST API,
    never through the raw engine.
    """
    engine = StorageEngine(engine_path)
    return {
        "memory_service": MemoryService(engine),
        "session_service": SessionService(engine),
        "agent_service": AgentService(engine),
        "skill_service": SkillService(engine),
        "analytics_service": AnalyticsService(engine),
        "export_service": ExportService(engine),
    }


def main():
    # Resolve engine path (same default as the API server)
    engine_path = os.environ.get("CONTEXTER_PATH", str(Path.home() / ".contexter"))

    # Create all service instances on the StorageEngine bridge.  Engine-open
    # failure (RocksDB LOCK, corrupt data, unwritable dir) becomes a clean
    # structured error + full server-side diagnostics + documented exit code.
    try:
        services = build_services(engine_path)
    except Exception as exc:
        _fail_engine_open(engine_path, exc)

    # Create the MCP server with live services
    mcp = create_mcp_server(**services)
    if mcp is None:
        print("Error: fastmcp not installed", file=sys.stderr)
        sys.exit(1)

    # Run with stdio transport (default for MCP subprocess)
    # If --sse flag is passed, run as SSE HTTP server instead
    if "--sse" in sys.argv:
        port = int(os.environ.get("CONTEXTER_MCP_PORT", "8052"))
        print(f"Starting Contexter MCP server (SSE) on port {port}", file=sys.stderr)
        mcp.run(transport="sse", port=port)
    else:
        mcp.run(transport="stdio")


if __name__ == "__main__":
    main()
