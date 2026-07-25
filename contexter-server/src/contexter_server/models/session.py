"""Session domain models."""

from datetime import datetime, timezone
from typing import Optional
from uuid import UUID, uuid4

from pydantic import BaseModel, Field


class Session(BaseModel):
    """A session represents a conversation or interaction with an agent."""

    id: UUID = Field(default_factory=uuid4)
    agent_id: UUID
    project: str = Field(..., min_length=1, max_length=256)
    name: Optional[str] = Field(None, max_length=512)
    status: str = Field(default="active")  # active, paused, completed, archived
    started_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
    updated_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
    completed_at: Optional[datetime] = None
    metadata: dict = Field(default_factory=dict)


class SessionCreate(BaseModel):
    """Input model for creating a new session."""

    agent_id: UUID
    project: str = Field(..., min_length=1, max_length=256)
    name: Optional[str] = Field(None, max_length=512)
    status: str = Field(default="active")
    metadata: dict = Field(default_factory=dict)


class SessionPatch(BaseModel):
    """Input model for partially updating a session."""

    name: Optional[str] = Field(None, max_length=512)
    status: Optional[str] = None  # active, paused, completed, archived
    metadata: Optional[dict] = None


class SessionFilter(BaseModel):
    """Filter criteria for listing sessions."""

    project: Optional[str] = None
    status: Optional[str] = None
