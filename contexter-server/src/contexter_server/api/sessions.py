"""FastAPI router for Session CRUD and resume operations."""

from fastapi import APIRouter, Depends, HTTPException, status

from contexter_server.models.session import Session, SessionCreate, SessionFilter, SessionPatch
from contexter_server.services.session_service import SessionService
from .deps import _validate_id_length, get_session_service


router = APIRouter(prefix="/api/v1/sessions", tags=["sessions"])


@router.get("", response_model=list[Session])
async def list_sessions(
    project: str | None = None,
    status_filter: str | None = None,
    service: SessionService = Depends(get_session_service),
) -> list[Session]:
    """List all sessions, optionally filtered by project or status."""
    return await service.list(SessionFilter(project=project, status=status_filter))


@router.post("", response_model=Session, status_code=status.HTTP_201_CREATED)
async def create_session(
    data: SessionCreate,
    service: SessionService = Depends(get_session_service),
) -> Session:
    """Create a new session."""
    try:
        return await service.create(data)
    except ValueError as exc:
        raise HTTPException(
            status_code=status.HTTP_409_CONFLICT,
            detail=str(exc),
        ) from exc


@router.get("/{id}", response_model=Session)
async def get_session(
    id: str,
    service: SessionService = Depends(get_session_service),
) -> Session:
    """Get a session by ID."""
    _validate_id_length(id)
    session = await service.get(id)
    if not session:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Session not found",
        )
    return session


@router.put("/{id}", response_model=Session)
async def update_session(
    patch: SessionPatch,
    id: str,
    service: SessionService = Depends(get_session_service),
) -> Session:
    """Update a session by ID (partial update)."""
    _validate_id_length(id)
    session = await service.update(id, patch)
    if not session:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Session not found",
        )
    return session


@router.delete("/{id}", status_code=status.HTTP_204_NO_CONTENT)
async def delete_session(
    id: str,
    service: SessionService = Depends(get_session_service),
) -> None:
    """Delete a session by ID (idempotent)."""
    _validate_id_length(id)
    await service.delete(id)


@router.post("/{id}/resume", response_model=Session)
async def resume_session(
    id: str,
    service: SessionService = Depends(get_session_service),
) -> Session:
    """Resume a paused or completed session."""
    _validate_id_length(id)
    session = await service.resume(id)
    if not session:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Session not found",
        )
    return session
