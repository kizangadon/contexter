#!/usr/bin/env python3
"""Launcher for the Contexter MCP server (stdio transport).

Spawns the FastMCP server with all services wired to the storage engine.
OpenCode's MCP config runs this script as a subprocess.

Usage:
    python3 run_mcp.py          # Stdio transport (for OpenCode MCP)
    python3 run_mcp.py --sse    # SSE transport on port 8052
"""

import sys
import os
from pathlib import Path

# Ensure contexter-server is on the path
sys.path.insert(0, str(Path(__file__).resolve().parent / "src"))

from contexter_server.mcp_server import create_mcp_server
from contexter_server.services import (
    MemoryService,
    SessionService,
    AgentService,
    SkillService,
    AnalyticsService,
    ExportService,
)
from contexter_core import Engine


def main():
    # Resolve engine path (same default as the API server)
    engine_path = os.environ.get("CONTEXTER_PATH", str(Path.home() / ".contexter"))

    # Open the storage engine
    engine = Engine.open(engine_path)

    # Create all service instances
    services = {
        "memory_service": MemoryService(engine),
        "session_service": SessionService(engine),
        "agent_service": AgentService(engine),
        "skill_service": SkillService(engine),
        "analytics_service": AnalyticsService(engine),
        "export_service": ExportService(engine),
    }

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
