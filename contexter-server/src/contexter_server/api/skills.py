"""FastAPI router for Skill CRUD operations."""

from fastapi import APIRouter, Depends, HTTPException, status

from contexter_server.models.skill import Skill, SkillCreate, SkillPatch
from contexter_server.services.skill_service import SkillService

from .deps import get_skill_service

router = APIRouter(prefix="/api/v1/skills", tags=["skills"])


@router.get("", response_model=list[Skill])
async def list_skills(
    service: SkillService = Depends(get_skill_service),
) -> list[Skill]:
    """List all skills."""
    return await service.list()


@router.post("", response_model=Skill, status_code=status.HTTP_201_CREATED)
async def create_skill(
    data: SkillCreate,
    service: SkillService = Depends(get_skill_service),
) -> Skill:
    """Create a new skill definition."""
    return await service.create(data)


@router.get("/{id}", response_model=Skill)
async def get_skill(
    id: str,
    service: SkillService = Depends(get_skill_service),
) -> Skill:
    """Get a skill by ID."""
    skill = await service.get(id)
    if not skill:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Skill not found",
        )
    return skill


@router.put("/{id}", response_model=Skill)
async def update_skill(
    id: str,
    patch: SkillPatch,
    service: SkillService = Depends(get_skill_service),
) -> Skill:
    """Update a skill by ID (partial update)."""
    skill = await service.update(id, patch)
    if not skill:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Skill not found",
        )
    return skill


@router.delete("/{id}", status_code=status.HTTP_204_NO_CONTENT)
async def delete_skill(
    id: str,
    service: SkillService = Depends(get_skill_service),
) -> None:
    """Delete a skill by ID (idempotent)."""
    await service.delete(id)
