"""FastAPI dependency injection for Contexter domain services.

Each ``get_*_service`` function returns a callable dependency that FastAPI
resolves per-request. Services are created once at startup and stored on the
app's ``state`` object by ``main.py``.
"""

import hmac
import os
from collections.abc import AsyncIterator
from typing import Any, TypeVar

from fastapi import HTTPException, Request, status
from structlog import get_logger

from contexter_server.services import (
    AgentService,
    AnalyticsService,
    AuditService,
    CorrelationService,
    ExportService,
    MemoryService,
    NotificationService,
    OnboardingService,
    SearchService,
    SessionService,
    SettingsService,
    SkillService,
)

_logger = get_logger(__name__)


# ---------------------------------------------------------------------------
# API Key Authentication
# ---------------------------------------------------------------------------


async def get_api_key(request: Request) -> None:
    """Validate ``Authorization: Bearer <key>`` against ``CONtexTER_API_KEY``.

    When the environment variable ``CONtexTER_API_KEY`` is set, every
    request *must* carry a matching ``Authorization: Bearer <key>`` header.
    Requests without a header or with a wrong value receive a ``401``
    response.

    When the variable is **not** set the check is skipped with a warning
    log — this preserves backward compatibility for development and
    environments that do not require API key auth.
    """
    api_key = os.environ.get("CONtexTER_API_KEY", "")
    if not api_key:
        _logger.warning(
            "CONtexTER_API_KEY not set — API key authentication is DISABLED"
        )
        return
    auth_header = request.headers.get("Authorization", "")
    if not auth_header.startswith("Bearer "):
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Missing or invalid Authorization header",
        )
    token = auth_header.removeprefix("Bearer ")
    if not hmac.compare_digest(token, api_key):
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Invalid API key",
        )


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


S = TypeVar("S")


def _get_service(request: Request, attr: str) -> S:  # type: ignore[type-arg]
    """Retrieve a service instance from ``request.app.state``."""
    service = getattr(request.app.state, attr, None)
    if service is None:
        msg = f"Service '{attr}' not initialised — call create_app() with services"
        raise RuntimeError(msg)
    return service  # type: ignore[return-value]


# ---------------------------------------------------------------------------
# ID length validation
# ---------------------------------------------------------------------------


def _validate_id_length(id: str, max_length: int = 512) -> None:
    """Validate that a path parameter ID doesn't exceed maximum length."""
    if len(id) > max_length:
        raise HTTPException(
            status_code=status.HTTP_422_UNPROCESSABLE_ENTITY,
            detail=f"ID length exceeds maximum of {max_length}",
        )


# ---------------------------------------------------------------------------
# Session
# ---------------------------------------------------------------------------


async def get_session_service(request: Request) -> AsyncIterator[SessionService]:
    yield _get_service(request, "session_service")


# ---------------------------------------------------------------------------
# Memory
# ---------------------------------------------------------------------------


async def get_memory_service(request: Request) -> AsyncIterator[MemoryService]:
    yield _get_service(request, "memory_service")


# ---------------------------------------------------------------------------
# Agent
# ---------------------------------------------------------------------------


async def get_agent_service(request: Request) -> AsyncIterator[AgentService]:
    yield _get_service(request, "agent_service")


# ---------------------------------------------------------------------------
# Skill
# ---------------------------------------------------------------------------


async def get_skill_service(request: Request) -> AsyncIterator[SkillService]:
    yield _get_service(request, "skill_service")


# ---------------------------------------------------------------------------
# Analytics
# ---------------------------------------------------------------------------


async def get_analytics_service(
    request: Request,
) -> AsyncIterator[AnalyticsService]:
    yield _get_service(request, "analytics_service")


# ---------------------------------------------------------------------------
# Search
# ---------------------------------------------------------------------------


async def get_search_service(request: Request) -> AsyncIterator[SearchService]:
    yield _get_service(request, "search_service")


# ---------------------------------------------------------------------------
# Settings
# ---------------------------------------------------------------------------


async def get_settings_service(
    request: Request,
) -> AsyncIterator[SettingsService]:
    yield _get_service(request, "settings_service")


# ---------------------------------------------------------------------------
# Notifications
# ---------------------------------------------------------------------------


async def get_notification_service(
    request: Request,
) -> AsyncIterator[NotificationService]:
    yield _get_service(request, "notification_service")


# ---------------------------------------------------------------------------
# Audit
# ---------------------------------------------------------------------------


async def get_audit_service(request: Request) -> AsyncIterator[AuditService]:
    yield _get_service(request, "audit_service")


# ---------------------------------------------------------------------------
# Correlation
# ---------------------------------------------------------------------------


async def get_correlation_service(
    request: Request,
) -> AsyncIterator[CorrelationService]:
    yield _get_service(request, "correlation_service")


# ---------------------------------------------------------------------------
# Export
# ---------------------------------------------------------------------------


async def get_export_service(request: Request) -> AsyncIterator[ExportService]:
    yield _get_service(request, "export_service")


# ---------------------------------------------------------------------------
# Onboarding
# ---------------------------------------------------------------------------


async def get_onboarding_service(
    request: Request,
) -> AsyncIterator[OnboardingService]:
    yield _get_service(request, "onboarding_service")
