"""MCP tool and resource authentication helpers.

Validates API key access for FastMCP tools and resources using the same
``CONtexTER_API_KEY`` environment variable as the FastAPI REST layer.
"""

import hmac
import os

from structlog import get_logger

_logger = get_logger(__name__)


class MCPAuthError(ValueError):
    """Raised when an MCP tool or resource call lacks valid authentication.

    Inherits from ``ValueError`` so that FastMCP serialises it as a clean
    JSON-RPC error rather than an internal server fault.
    """

    pass


def require_api_key(api_key: str | None = None) -> None:
    """Validate an API key for MCP tool or resource access.

    When the environment variable ``CONtexTER_API_KEY`` is set, the
    caller *must* supply a matching ``api_key`` value.  When the variable
    is **not** set the check is skipped — this preserves backward
    compatibility for development and environments that do not require
    API key authentication.

    Parameters
    ----------
    api_key:
        The API key provided by the caller, or ``None`` / empty if none
        was given.

    Raises
    ------
    MCPAuthError
        If ``CONtexTER_API_KEY`` is set and ``api_key`` does not match.
    """
    expected = os.environ.get("CONtexTER_API_KEY", "")
    if not expected:
        return  # No auth configured — allow access.

    if not api_key:
        _logger.warning("mcp_tool.auth.missing_api_key")
        raise MCPAuthError(
            "API key required. Provide a matching _api_key parameter "
            "or unset CONtexTER_API_KEY to disable authentication."
        )

    if not hmac.compare_digest(api_key, expected):
        _logger.warning("mcp_tool.auth.invalid_api_key")
        raise MCPAuthError("Invalid API key.")
