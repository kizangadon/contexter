"""FastMCP framework logging policy: bounded failure stderr (REQ-FL-001).

FastMCP's generic ``except Exception`` paths
(``fastmcp/server/server.py``) run ``logger.exception`` for every
tool/resource/prompt error, which renders a 2672-char rich traceback box
(with source frames) on stderr via the namespace's traceback
RichHandler (``fastmcp.utilities.logging.configure_logging``).  The
sampling path (``fastmcp/server/sampling/run.py``) and the prompt
function path (``fastmcp/prompts/function_prompt.py``) emit the same
class of record on their own module loggers, and the schema-validation
path (``fastmcp/server/server.py:1290``) emits a WARNING with
``exc_info`` plus a file:line reference — all of it must be kept off
failure stderr.

This module installs a ``logging.Filter`` directly on the ``fastmcp``
logger namespace.  Because FastMCP sets ``propagate=False`` on that
namespace (EC-FL-001), root-logger configuration never reaches it — the
filter must target the namespace itself.  Python's logging applies only
the ORIGINATING logger's filters, so every emitter logger must carry
the filter explicitly (EC-FC-001).  Filters survive FastMCP's
``configure_logging`` (which removes handlers only), so installation is
safe at any point after ``fastmcp`` is first imported.

Drop-policy (REQ-FC-005): covered framework error/warning records are
dropped at EVERY level — ERROR ``logger.exception`` paths, the
schema-validation WARNING at server.py:1290, and the sampling
``logger.log(e.log_level, ...)`` path where ``e.log_level`` may be any
level.  Downgrading the record's level is NOT sufficient: RichHandler
still renders the ``exc_info`` traceback (a long resource URI wraps the
line to ~583 bytes measured), so the drop is the contract.  Covered
records contribute ZERO bytes to failure stderr: the only remaining
output is contexter's own bounded structured lines
(``handler_error`` / ``bridge_call_failed``), which stay <=512 chars
total (REQ-FL-001).  Matching is explicit per-prefix via
``startswith`` (EC-FC-002) — contexter's own record messages
(``bridge_call_failed``, ``handler_error``) never collide with a
prefix.  Client-visible error frames are untouched (REQ-FL-002), full
tracebacks remain in the bridge diagnostics log (REQ-FL-003), and
success-path/unrelated records pass through unchanged (REQ-FL-004).
"""

import logging

# Framework error-call record messages whose rich traceback box must be
# suppressed.  Coverage is pinned by
# tests/mcp/test_framework_efs_coverage.py::TestEmitterInventoryDrift
# against a live inventory of the installed fastmcp package (REQ-FC-004):
#   - "Error calling tool "          fastmcp/server/server.py:1285,1297
#   - "Error calling sampling tool " fastmcp/server/sampling/run.py:322,336
#   - "Error reading resource "      fastmcp/server/server.py:1423,1428,1431
#   - "Error rendering prompt "      fastmcp/server/server.py:1587-1594,
#                                    fastmcp/prompts/function_prompt.py:370
#   - "Invalid arguments for tool "  fastmcp/server/server.py:1291 (WARNING,
#                                    schema-validation, exc_info + file:line)
_FRAMEWORK_ERROR_PREFIXES = (
    "Error calling tool ",
    "Error calling sampling tool ",
    "Error reading resource ",
    "Error rendering prompt ",
    "Invalid arguments for tool ",
)

_INSTALLED_ATTR = "_contexter_bounded_stderr_filter_installed"

# Loggers that emit (or may emit) framework error-call records.
# fastmcp's ``get_logger(__name__)`` resolves to the module's dotted
# name (``fastmcp.server.server``, ``fastmcp.prompts.function_prompt``,
# ``fastmcp.server.sampling.run``), so error records originate there.
# Python's logging applies only the ORIGINATING logger's filters, so
# the filter must be installed on every emitter (and the namespace
# root), not merely on the parent.  ``logging.getLogger`` on these
# names before FastMCP imports is harmless: FastMCP later resolves the
# same singletons, and its ``configure_logging`` removes handlers only
# (filters survive).
_EMITTER_LOGGERS = (
    "fastmcp",
    "fastmcp.server",
    "fastmcp.server.server",
    "fastmcp.prompts.function_prompt",
    "fastmcp.server.sampling.run",
)


class _SuppressFrameworkTracebackBox(logging.Filter):
    """Suppress framework error-call records (no stderr output for failures).

    FastMCP renders a 2672-char rich traceback box on stderr for any
    error-call record with ``exc_info`` set, and even its downgraded
    form can exceed the budget when it renders long payloads (a resource
    URI wraps the RichHandler line to ~583 bytes measured — EC-FC-005).
    Dropping the framework error-call records at EVERY level (REQ-FC-005)
    guarantees the framework contributes ZERO bytes to failure stderr:
    the only remaining output is contexter's own bounded structured
    lines (``handler_error`` / ``bridge_call_failed``), which stay
    <=512 chars total (REQ-FL-001).  Client-visible error frames are
    untouched (REQ-FL-002), full tracebacks remain in the bridge
    diagnostics log (REQ-FL-003), and success-path/unrelated records
    pass through unchanged (REQ-FL-004).
    """

    def filter(self, record: logging.LogRecord) -> bool:
        if record.getMessage().startswith(_FRAMEWORK_ERROR_PREFIXES):
            return False
        return True


def configure_fastmcp_failure_stderr() -> None:
    """Install the bounded-stderr filter on the FastMCP emitter loggers.

    Idempotent: the filter is installed at most once per logger.  Must
    target the ``fastmcp`` namespace directly because FastMCP configures
    ``propagate=False`` on it (EC-FL-001), and must include the emitter
    loggers because Python's logging applies only the originating
    logger's filters.
    """
    for name in _EMITTER_LOGGERS:
        logger = logging.getLogger(name)
        if getattr(logger, _INSTALLED_ATTR, False):
            continue
        logger.addFilter(_SuppressFrameworkTracebackBox())
        setattr(logger, _INSTALLED_ATTR, True)
