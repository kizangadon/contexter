"""Audit domain models for tracking entity changes."""

from datetime import datetime, timezone
from typing import Any, Optional
from uuid import UUID, uuid4

from pydantic import BaseModel, Field


class AuditEntry(BaseModel):
    """An audit trail entry recording an action on an entity."""

    id: UUID = Field(default_factory=uuid4)
    entity_type: str
    entity_id: str
    action: str  # created, updated, deleted, accessed
    actor: Optional[str] = None
    timestamp: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
    details: dict[str, Any] = Field(default_factory=dict)
    previous_value: Optional[str] = None
    new_value: Optional[str] = None


class AuditFilter(BaseModel):
    """Filter criteria for querying audit entries."""

    entity_type: Optional[str] = None
    action: Optional[str] = None
    actor: Optional[str] = None
    query: Optional[str] = None
    limit: int = Field(default=50, ge=1, le=500)
    offset: int = Field(default=0, ge=0)
