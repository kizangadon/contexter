"""Session domain models."""

from datetime import datetime, timezone
from typing import Optional
from uuid import UUID, uuid4

from pydantic import AliasChoices, BaseModel, Field, ConfigDict, field_validator


class Session(BaseModel):
    """A session represents a conversation or interaction with an agent."""

    model_config = ConfigDict(populate_by_name=True)
    # Accept camelCase from Rust (via validation_alias) AND snake_case from Python code (by field name)

    id: UUID = Field(default_factory=uuid4)
    agent_id: Optional[UUID] = Field(default=None, validation_alias=AliasChoices("agent_id", "agentId"))
    project: str = Field(..., min_length=1, max_length=256)
    name: Optional[str] = Field(None, max_length=512)
    status: str = Field(default="active")  # active, paused, completed, archived
    turn_count: int = Field(default=0, validation_alias="turnCount")
    duration_ms: int = Field(default=0, validation_alias="durationMs")
    efficiency_score: Optional[float] = Field(
        default=None, validation_alias="efficiencyScore"
    )
    started_at: datetime = Field(
        default_factory=lambda: datetime.now(timezone.utc),
        validation_alias="createdAt",
    )
    updated_at: datetime = Field(
        default_factory=lambda: datetime.now(timezone.utc),
    )
    last_active: datetime = Field(
        default_factory=lambda: datetime.now(timezone.utc),
        validation_alias="lastActive",
    )
    completed_at: Optional[datetime] = None
    metadata: dict = Field(default_factory=dict)

    @field_validator('started_at', 'updated_at', 'last_active', 'completed_at', mode='before')
    @classmethod
    def ensure_utc(cls, v):
        if isinstance(v, datetime) and v.tzinfo is None:
            return v.replace(tzinfo=timezone.utc)
        return v

    @field_validator('status', mode='before')
    @classmethod
    def normalize_status(cls, v):
        if v == 'done':
            return 'completed'
        return v


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
