"""FastAPI application factory for the Contexter REST API server.

Creates the ``FastAPI`` application with:
- Lifespan management (StorageEngine + all services on startup, flush on shutdown)
- Logging middleware (method, path, status, duration for all requests)
- Security middleware (API key auth, security headers, body size limiting, etc.)
- Health endpoint ``GET /health``
- All ``/api/v1/`` routers registered
"""

import os
import threading
import time
from collections.abc import AsyncIterator
from contextlib import asynccontextmanager
from typing import Any

from fastapi import Depends, FastAPI, Request
from fastapi.responses import JSONResponse
from slowapi.extension import Limiter as SlowApiLimiter
from slowapi.middleware import SlowAPIMiddleware
from starlette.middleware.trustedhost import TrustedHostMiddleware
from structlog import get_logger

from contexter_server.rate_limiter import create_limiter

from contexter_server.api import (
    agents,
    analytics,
    audit,
    changelog,
    correlation,
    efficiency,
    export,
    feedback,
    files,
    memories,
    notifications,
    onboarding,
    search,
    sessions,
    settings,
    skills,
)
from contexter_server.api.deps import get_api_key
from contexter_server.core.bridge import StorageEngine
from contexter_server.mcp_server import create_mcp_server
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

_MCP_PORT = 8052

logger = get_logger(__name__)


def _run_mcp_server(mcp: Any, shutdown_event: threading.Event | None = None) -> None:
    """Run the MCP server with SSE transport in a blocking daemon thread.

    Parameters
    ----------
    mcp:
        The FastMCP server instance to run.
    shutdown_event:
        Optional ``threading.Event`` that, when set, signals the server to
        stop. If provided the server will attempt a graceful shutdown.
    """
    try:
        mcp.run(transport="sse", port=_MCP_PORT)
    except Exception:
        logger.exception("mcp_server.failed")


def _create_services(engine: StorageEngine) -> dict:
    """Create all domain service instances.

    Returns a dict keyed by attribute name (e.g. ``"session_service"``)
    suitable for setting as ``app.state.*`` attributes.
    """
    return {
        "session_service": SessionService(engine),
        "memory_service": MemoryService(engine),
        "agent_service": AgentService(engine),
        "skill_service": SkillService(engine),
        "analytics_service": AnalyticsService(engine),
        "search_service": SearchService(engine),
        "settings_service": SettingsService(engine),
        "notification_service": NotificationService(engine),
        "audit_service": AuditService(engine),
        "correlation_service": CorrelationService(engine),
        "export_service": ExportService(engine),
        "onboarding_service": OnboardingService(engine),
    }


def _register_routers(app: FastAPI) -> None:
    """Register all API route routers on the application.

    Every ``/api/v1/`` router gets the ``get_api_key`` dependency so that
    API-key authentication is enforced unless ``CONtexTER_API_KEY`` is unset.
    """
    router_auth = [Depends(get_api_key)]
    for router in (
        sessions.router,
        memories.router,
        agents.router,
        skills.router,
        analytics.router,
        efficiency.router,
        search.router,
        settings.router,
        notifications.router,
        audit.router,
        files.router,
        correlation.router,
        export.router,
        feedback.router,
        onboarding.router,
        changelog.router,
    ):
        app.include_router(router, dependencies=router_auth)


def _add_logging_middleware(app: FastAPI) -> None:
    """Add request logging middleware using structlog.

    Logs every request with: method, path, status code, duration_ms.
    """

    @app.middleware("http")
    async def log_requests(request: Request, call_next):  # type: ignore[no-untyped-def]
        start = time.monotonic()
        response = await call_next(request)
        duration_ms = (time.monotonic() - start) * 1000

        logger.info(
            "http_request",
            method=request.method,
            path=request.url.path,
            status=response.status_code,
            duration_ms=round(duration_ms, 2),
        )
        return response


def _add_security_headers_middleware(app: FastAPI) -> None:
    """Add security headers to every HTTP response.

    Sets:
    - ``X-Content-Type-Options: nosniff``
    - ``X-Frame-Options: DENY``
    - ``Content-Security-Policy: default-src 'self'``
    - ``Referrer-Policy: no-referrer``
    """

    @app.middleware("http")
    async def add_security_headers(  # type: ignore[no-untyped-def]
        request: Request,
        call_next,
    ) -> JSONResponse:
        response = await call_next(request)
        response.headers["X-Content-Type-Options"] = "nosniff"
        response.headers["X-Frame-Options"] = "DENY"
        response.headers["Content-Security-Policy"] = "default-src 'self'"
        response.headers["Referrer-Policy"] = "no-referrer"
        return response


def _add_body_size_limit_middleware(app: FastAPI) -> None:
    """Reject requests whose body exceeds ``MAX_REQUEST_BODY``.

    The limit is read from the ``MAX_REQUEST_BODY`` environment variable
    (in bytes) and defaults to 1 048 576 (1 MiB).

    ``Transfer-Encoding: chunked`` is explicitly rejected because
    FastAPI/Starlette streams chunked bodies and the ``Content-Length``
    header — if present — may not reflect the actual payload size.
    """

    @app.middleware("http")
    async def limit_body_size(  # type: ignore[no-untyped-def]
        request: Request,
        call_next,
    ) -> JSONResponse:
        # Reject chunked transfer encoding — we cannot rely on
        # Content-Length to bound the body size.
        transfer_encoding = request.headers.get("Transfer-Encoding", "")
        if "chunked" in transfer_encoding.lower():
            return JSONResponse(
                status_code=413,
                content={"detail": "Transfer-Encoding chunked not supported"},
            )

        max_bytes = int(os.environ.get("MAX_REQUEST_BODY", str(1 * 1024 * 1024)))
        content_length_str = request.headers.get("Content-Length")
        if content_length_str is not None:
            try:
                content_length = int(content_length_str)
            except ValueError:
                content_length = 0
            if content_length > max_bytes:
                return JSONResponse(
                    status_code=413,
                    content={"detail": "Request body too large"},
                )
        return await call_next(request)


def _add_rate_limiting_middleware(app: FastAPI) -> SlowApiLimiter:
    """Add rate limiting middleware via slowapi.

    The limiter is created by ``create_limiter()`` which reads the
    ``CONtexTER_RATE_LIMIT_ENABLED`` and ``CONtexTER_RATE_LIMIT``
    environment variables.

    Returns the ``Limiter`` instance so callers can attach route-level
    rate-limit decorators or mark routes as exempt.
    """
    limiter, (exc_cls, handler) = create_limiter()
    app.state.limiter = limiter
    app.add_middleware(SlowAPIMiddleware)
    app.add_exception_handler(exc_cls, handler)
    return limiter


def _resolve_docs_url() -> tuple[str | None, str | None, str | None]:
    """Return (docs_url, redoc_url, openapi_url) based on config.

    When ``CONtexTER_ENABLE_DOCS=true`` the interactive docs and OpenAPI
    schema are served; otherwise they are disabled (return ``None``).
    """
    enable = os.environ.get("CONtexTER_ENABLE_DOCS", "").strip().lower() == "true"
    if enable:
        return "/docs", "/redoc", "/openapi.json"
    logger.info("OpenAPI docs disabled (set CONtexTER_ENABLE_DOCS=true to enable)")
    return None, None, None


def create_app(data_path: str = "~/.contexter/") -> FastAPI:
    """Create and return the Contexter FastAPI application.

    Parameters
    ----------
    data_path:
        Filesystem path passed to ``StorageEngine`` for data persistence.
        Expanded with ``~`` → user home directory.
    """

    @asynccontextmanager
    async def lifespan(app: FastAPI) -> AsyncIterator[None]:
        """Initialise bridge + services on startup; flush on shutdown."""
        logger.info("contexter_server.starting", data_path=data_path)
        engine = StorageEngine(data_path)

        services = _create_services(engine)
        for attr, svc in services.items():
            setattr(app.state, attr, svc)

        app.state.storage_engine = engine

        # Start MCP server (SSE transport, port 8052) in a daemon thread
        # so it is cleaned up when the main process exits.
        mcp_services = {
            "memory_service": services["memory_service"],
            "session_service": services["session_service"],
            "agent_service": services["agent_service"],
            "skill_service": services["skill_service"],
            "analytics_service": services["analytics_service"],
            "export_service": services["export_service"],
        }
        mcp = create_mcp_server(**mcp_services)
        mcp_shutdown_event: threading.Event | None = None
        mcp_thread: threading.Thread | None = None
        if mcp is not None:
            mcp_shutdown_event = threading.Event()
            mcp_thread = threading.Thread(
                target=_run_mcp_server,
                args=(mcp, mcp_shutdown_event),
                daemon=True,
            )
            mcp_thread.start()
            logger.info("mcp_server.started", port=_MCP_PORT)
        else:
            logger.warning("mcp_server.not_available")

        # Expose thread and shutdown event so they can be inspected/overridden
        # in tests or during controlled shutdown.
        app.state.mcp_thread = mcp_thread
        app.state.mcp_shutdown_event = mcp_shutdown_event

        logger.info("contexter_server.started", services=list(services.keys()))
        try:
            yield
        finally:
            logger.info("contexter_server.shutting_down_mcp")
            if mcp_shutdown_event is not None and mcp_thread is not None:
                mcp_shutdown_event.set()
                mcp_thread.join(timeout=5.0)
                if mcp_thread.is_alive():
                    logger.warning("mcp_server.did_not_shutdown_gracefully")
                else:
                    logger.info("mcp_server.stopped")

            logger.info("contexter_server.flushing")
            try:
                await engine.flush()
            except Exception:
                logger.exception("contexter_server.flush_error")
            logger.info("contexter_server.stopped")

    docs_url, redoc_url, openapi_url = _resolve_docs_url()
    app = FastAPI(
        title="Contexter API",
        version="0.1.0",
        lifespan=lifespan,
        debug=False,
        docs_url=docs_url,
        redoc_url=redoc_url,
        openapi_url=openapi_url,
    )

    _register_routers(app)
    _add_logging_middleware(app)
    _add_security_headers_middleware(app)
    _add_body_size_limit_middleware(app)
    limiter = _add_rate_limiting_middleware(app)
    app.add_middleware(
        TrustedHostMiddleware,
        allowed_hosts=["127.0.0.1", "localhost"],
    )

    @app.get("/health")
    @limiter.exempt
    async def health() -> dict:
        """Health check endpoint — returns 200 when the service is alive."""
        return {"status": "ok"}

    return app


app = create_app()
