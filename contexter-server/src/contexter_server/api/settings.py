"""FastAPI router for system configuration and settings."""

from fastapi import APIRouter, Depends, HTTPException, status

from contexter_server.models.settings import SectionUpdate
from contexter_server.services.settings_service import SettingsService

from .deps import get_settings_service

router = APIRouter(prefix="/api/v1/settings", tags=["settings"])


@router.get("/{section}", response_model=dict)
async def get_section(
    section: str,
    service: SettingsService = Depends(get_settings_service),
) -> dict:
    """Get a single configuration section by name."""
    result = await service.get_section(section)
    if result is None:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail=f"Settings section '{section}' not found",
        )
    return result


@router.put("/{section}", response_model=dict)
async def update_section(
    section: str,
    body: SectionUpdate,
    service: SettingsService = Depends(get_settings_service),
) -> dict:
    """Update a single configuration section using typed key-value pairs."""
    await service.update_section(section, body.values)
    updated = await service.get_section(section)
    if updated is None:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail=f"Settings section '{section}' not found",
        )
    return updated
