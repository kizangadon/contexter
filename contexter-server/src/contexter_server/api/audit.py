"""FastAPI router for audit trail queries."""

from fastapi import APIRouter, Depends, Query

from contexter_server.models.audit import AuditEntry, AuditFilter
from contexter_server.services.audit_service import AuditService

from .deps import get_audit_service

router = APIRouter(prefix="/api/v1/audit", tags=["audit"])


@router.get("", response_model=list[AuditEntry])
async def query_audit(
    entity_type: str | None = Query(None, description="Filter by entity type"),
    action: str | None = Query(None, description="Filter by action (created, updated, deleted)"),
    actor: str | None = Query(None, description="Filter by actor ID"),
    q: str | None = Query(None, alias="q", description="Full-text search query"),
    limit: int = Query(50, ge=1, le=500, description="Max results"),
    offset: int = Query(0, ge=0, description="Result offset for pagination"),
    service: AuditService = Depends(get_audit_service),
) -> list[AuditEntry]:
    """Query audit trail entries matching the given filters."""
    filter_data = AuditFilter(
        entity_type=entity_type,
        action=action,
        actor=actor,
        query=q,
        limit=limit,
        offset=offset,
    )
    return await service.query(filter_data)
