"""Structured handler errors and shared validation bounds.

The MCP handler layer never smuggles ``{"error": ...}`` payloads inside
success frames. Every failure path raises :class:`HandlerError`, which the
FastMCP wrapper serialises as a structured MCP error (``isError`` result
frame for tools, protocol error for resources) — preserving the frozen
contract (REQ-007/AC-6/EC-001) and the ``Resource not found: <id>``
message convention.

The class inherits from ``ValueError`` (like ``MCPAuthError``) so that
FastMCP's exception-to-error-frame conversion keeps the message intact.
"""

MAX_CONTENT_LENGTH = 1_000_000
MAX_QUERY_LENGTH = 10_000
MAX_LIST_LIMIT = 100
MAX_SEARCH_LIMIT = 100
DEFAULT_SEARCH_LIMIT = 20

EXPORT_FORMATS = frozenset({"json", "yaml", "csv"})
DEFAULT_EXPORT_FORMAT = "json"


class HandlerError(ValueError):
    """A structured, client-visible handler error.

    Parameters
    ----------
    kind:
        Machine-readable category: ``not_found``, ``validation``,
        ``storage``, ``service_unavailable``.
    message:
        Client-visible message. Bounded — never echoes unbounded input.
    """

    def __init__(self, kind: str, message: str) -> None:
        super().__init__(message)
        self.kind = kind
        self.message = message


def not_found_error(resource: str) -> HandlerError:
    """Frozen not-found convention: ``Resource not found: <id>``."""
    return HandlerError("not_found", f"Resource not found: {resource}")


def validation_error(message: str) -> HandlerError:
    return HandlerError("validation", message)


def storage_error(message: str) -> HandlerError:
    return HandlerError("storage", message)
