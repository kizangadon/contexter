"""Skill domain models."""

from datetime import datetime, timezone
from typing import Optional
from uuid import UUID, uuid4

from pydantic import BaseModel, Field


class Skill(BaseModel):
    """A skill defines a capability an agent can use."""

    id: UUID = Field(default_factory=uuid4)
    name: str = Field(..., min_length=1, max_length=256)
    description: Optional[str] = Field(None, max_length=1024)
    type: str  # memory, search, reasoning, custom
    version: Optional[str] = None
    parameters: dict = Field(default_factory=dict)
    enabled: bool = Field(default=True)
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
    updated_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))


class SkillCreate(BaseModel):
    """Input model for creating a new skill."""

    name: str = Field(..., min_length=1, max_length=256)
    description: Optional[str] = Field(None, max_length=1024)
    type: str
    version: Optional[str] = None
    parameters: dict = Field(default_factory=dict)
    enabled: bool = Field(default=True)


class SkillPatch(BaseModel):
    """Input model for partially updating a skill."""

    name: Optional[str] = Field(None, max_length=256)
    description: Optional[str] = Field(None, max_length=1024)
    type: Optional[str] = None
    version: Optional[str] = None
    parameters: Optional[dict] = None
    enabled: Optional[bool] = None
