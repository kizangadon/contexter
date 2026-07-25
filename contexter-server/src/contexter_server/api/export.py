"""FastAPI router for data export operations."""

from fastapi import APIRouter, Depends, HTTPException, status

from contexter_server.models.export import ExportRequest, ExportStatus
from contexter_server.services.export_service import ExportService

from .deps import get_export_service

router = APIRouter(prefix="/api/v1/export", tags=["export"])


@router.post("/submit", response_model=ExportStatus, status_code=status.HTTP_201_CREATED)
async def submit_export(
    request: ExportRequest,
    service: ExportService = Depends(get_export_service),
) -> ExportStatus:
    """Submit a new data export request."""
    return await service.submit(request)


@router.get("/status/{id}", response_model=ExportStatus)
async def export_status(
    id: str,
    service: ExportService = Depends(get_export_service),
) -> ExportStatus:
    """Get the status of an export by ID."""
    status_result = await service.get_status(id)
    if status_result is None:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Export not found",
        )
    return status_result


@router.get("/download/{id}")
async def download_export(
    id: str,
    service: ExportService = Depends(get_export_service),
) -> dict:
    """Download the exported data for a completed export."""
    data = await service.download(id)
    if data is None:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Export not found or not yet completed",
        )
    return data


@router.get("/history", response_model=list[ExportStatus])
async def export_history(
    service: ExportService = Depends(get_export_service),
) -> list[ExportStatus]:
    """Get a list of recent export statuses."""
    return await service.history()
