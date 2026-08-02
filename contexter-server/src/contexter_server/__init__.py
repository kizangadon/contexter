"""Contexter Server — Python API layer for the Contexter agent memory system."""

import structlog

__version__ = "0.1.0"

# ---------------------------------------------------------------------------
# Structlog configuration
# ---------------------------------------------------------------------------
# Configure structlog with explicit processors so behaviour is predictable
# across environments.  JSON output with ISO timestamps is suitable for
# structured-log ingestion in both sync and async contexts.
#
# NOTE: we use LoggerFactory so structlog integrates with stdlib logging,
# which is necessary for structured log capture in testing (caplog) and
# for compatibility with async frameworks.  The root logger level is set
# to INFO below to ensure INFO+ messages are not silently dropped.
#
# TODO: configure async logging for high-throughput deployments.
# For standard workloads structlog's synchronous JSONRenderer is sufficient
# — the serialisation overhead per call is negligible.  If throughput exceeds
# ~10 000 log entries/second consider replacing with a non-blocking
# QueueHandler + QueueListener pattern (stdlib) or a dedicated async
# logging handler.

structlog.configure(
    processors=[
        structlog.stdlib.add_log_level,
        structlog.stdlib.add_logger_name,
        structlog.processors.TimeStamper(fmt="iso"),
        structlog.processors.StackInfoRenderer(),
        structlog.dev.ConsoleRenderer(),
    ],
    wrapper_class=structlog.stdlib.BoundLogger,
    context_class=dict,
    logger_factory=structlog.stdlib.LoggerFactory(),
    cache_logger_on_first_use=True,
)

# Ensure the root stdlib logger emits INFO+ so that structlog's stdlib
# integration does not silently swallow messages below WARNING (the
# default stdlib level).  We use INFO as the default because the
# codebase logs at INFO in normal operation.
import logging
_log = logging.getLogger()
_log.setLevel(logging.INFO)

# FastMCP framework logging policy: bound failure stderr (REQ-FL-001).
# Installed at package import so every entry point (run_mcp.py, API app,
# tests) gets the policy.  Filters survive FastMCP's own logging
# configuration because it removes handlers only.
from contexter_server.fastmcp_logging import configure_fastmcp_failure_stderr

configure_fastmcp_failure_stderr()
