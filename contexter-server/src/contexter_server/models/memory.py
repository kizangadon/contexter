"""Memory domain models."""

from datetime import datetime, timezone
from typing import Optional
from uuid import UUID, uuid4

from pydantic import AliasChoices, BaseModel, Field, ConfigDict, model_serializer, field_validator


class Memory(BaseModel):
    """A memory entry stored within a session."""

    model_config = ConfigDict(populate_by_name=True)
    # Accept camelCase from Rust (via validation_alias) AND snake_case from Python code (by field name)

    id: UUID = Field(default_factory=uuid4)
    session_id: Optional[UUID] = Field(default=None, validation_alias=AliasChoices("session_id", "sessionId"))
    agent_id: Optional[UUID] = Field(default=None, validation_alias=AliasChoices("agent_id", "agentId"))
    memory_type: str = Field(default="fact", validation_alias="memoryType")
    role: Optional[str] = Field(default="system")  # user, assistant, system, tool
    content: str
    embedding: Optional[list[float]] = None
    tags: list[str] = Field(default_factory=list)
    version: int = Field(default=1)
    tokens: Optional[int] = None
    tokenizer: Optional[str] = None
    model: Optional[str] = None
    created_at: datetime = Field(
        default_factory=lambda: datetime.now(timezone.utc),
        validation_alias="createdAt",
    )
    updated_at: datetime = Field(
        default_factory=lambda: datetime.now(timezone.utc),
        validation_alias="updatedAt",
    )
    metadata: dict = Field(default_factory=dict)

    @model_serializer(mode='wrap')
    def _serialize_without_embedding(self, handler):
        data = handler(self)
        data.pop('embedding', None)
        return data

    @field_validator('created_at', 'updated_at', mode='before')
    @classmethod
    def ensure_utc(cls, v):
        if isinstance(v, datetime) and v.tzinfo is None:
            return v.replace(tzinfo=timezone.utc)
        return v


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
