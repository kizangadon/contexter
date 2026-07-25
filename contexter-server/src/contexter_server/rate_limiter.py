"""Rate limiting configuration for the Contexter FastAPI application.

Provides a factory function that creates a ``slowapi.Limiter`` instance
whose behaviour is driven by environment variables.
"""

import os

from slowapi import Limiter, _rate_limit_exceeded_handler
from slowapi.errors import RateLimitExceeded
from slowapi.util import get_remote_address


def create_limiter() -> tuple[Limiter, tuple]:
    """Create and return a configured ``Limiter`` and its exception handler pair.

    Environment variables
    --------------------
    ``CONtexTER_RATE_LIMIT_ENABLED``
        Set to ``"false"`` to disable all rate limiting (default ``"true"``).
    ``CONtexTER_RATE_LIMIT``
        The default rate-limit string applied to every endpoint
        (default ``"100/minute"``).

    Returns
    -------
    tuple[Limiter, tuple]
        ``(limiter, (RateLimitExceeded, handler))`` — the limiter to set on
        ``app.state.limiter`` and the exception-handler pair to register via
        ``app.add_exception_handler``.
    """
    enabled = os.environ.get("CONtexTER_RATE_LIMIT_ENABLED", "true")
    enabled = enabled.strip().lower() != "false"

    limit_str = os.environ.get("CONtexTER_RATE_LIMIT", "100/minute")

    limiter = Limiter(
        key_func=get_remote_address,
        default_limits=[limit_str],
        enabled=enabled,
    )

    return limiter, (RateLimitExceeded, _rate_limit_exceeded_handler)
