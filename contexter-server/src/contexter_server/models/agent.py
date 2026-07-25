"""Agent domain models."""

from datetime import datetime, timezone
from typing import Optional
from uuid import UUID, uuid4

from pydantic import BaseModel, Field


class Agent(BaseModel):
    """An agent configuration with LLM provider settings."""

    id: UUID = Field(default_factory=uuid4)
    name: str = Field(..., min_length=1, max_length=256)
    provider: str  # openai, anthropic, ollama, custom
    model: str
    system_prompt: Optional[str] = None
    temperature: float = Field(default=0.7, ge=0.0, le=2.0)
    max_tokens: Optional[int] = Field(None, gt=0)
    tools: list[str] = Field(default_factory=list)
    metadata: dict = Field(default_factory=dict)
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
    updated_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))


class AgentCreate(BaseModel):
    """Input model for creating a new agent."""

    name: str = Field(..., min_length=1, max_length=256)
    provider: str
    model: str
    system_prompt: Optional[str] = None
    temperature: float = Field(default=0.7, ge=0.0, le=2.0)
    max_tokens: Optional[int] = Field(None, gt=0)
    tools: list[str] = Field(default_factory=list)
    metadata: dict = Field(default_factory=dict)


class AgentPatch(BaseModel):
    """Input model for partially updating an agent."""

    name: Optional[str] = Field(None, max_length=256)
    provider: Optional[str] = None
    model: Optional[str] = None
    system_prompt: Optional[str] = None
    temperature: Optional[float] = Field(None, ge=0.0, le=2.0)
    max_tokens: Optional[int] = Field(None, gt=0)
    tools: Optional[list[str]] = None
    metadata: Optional[dict] = None
