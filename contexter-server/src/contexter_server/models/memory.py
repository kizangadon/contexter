"""Memory domain models."""

from datetime import datetime, timezone
from typing import Optional
from uuid import UUID, uuid4

from pydantic import BaseModel, Field


class Memory(BaseModel):
    """A memory entry stored within a session."""

    id: UUID = Field(default_factory=uuid4)
    session_id: UUID
    agent_id: UUID
    role: str  # user, assistant, system, tool
    content: str
    tokens: Optional[int] = None
    tokenizer: Optional[str] = None
    model: Optional[str] = None
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
    metadata: dict = Field(default_factory=dict)


class MemoryCreate(BaseModel):
    """Input model for creating a new memory."""

    session_id: UUID
    agent_id: UUID
    role: str
    content: str
    tokens: Optional[int] = None
    tokenizer: Optional[str] = None
    model: Optional[str] = None
    metadata: dict = Field(default_factory=dict)


class MemoryPatch(BaseModel):
    """Input model for partially updating a memory."""

    content: Optional[str] = None
    tokens: Optional[int] = None
    metadata: Optional[dict] = None
