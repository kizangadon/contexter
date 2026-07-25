"""FastAPI router for first-run onboarding and setup wizard."""

from typing import Any

from fastapi import APIRouter, Depends
from pydantic import BaseModel, Field

from contexter_server.services.onboarding_service import OnboardingService

from .deps import get_onboarding_service

router = APIRouter(prefix="/api/v1/onboarding", tags=["onboarding"])


class WizardData(BaseModel):
    """Validated onboarding wizard configuration submission."""

    responses: dict[str, Any] = Field(
        ...,
        description="Key-value pairs of wizard responses (e.g. project_name)",
    )
    completed_step: str = Field(
        ...,
        min_length=1,
        description="Identifier of the last completed wizard step",
    )


@router.get("/status")
async def onboarding_status(
    service: OnboardingService = Depends(get_onboarding_service),
) -> dict:
    """Check if onboarding has been completed."""
    return await service.get_status()


@router.post("/wizard")
async def submit_wizard(
    body: WizardData,
    service: OnboardingService = Depends(get_onboarding_service),
) -> dict:
    """Save onboarding wizard configuration."""
    return await service.submit_wizard(body.responses)


@router.get("/progress")
async def onboarding_progress(
    service: OnboardingService = Depends(get_onboarding_service),
) -> dict:
    """Get onboarding completion progress."""
    return await service.get_progress()
