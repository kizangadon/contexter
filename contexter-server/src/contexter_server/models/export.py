"""Export domain models for data export operations."""

from datetime import datetime, timezone
from typing import Optional
from uuid import UUID

from pydantic import BaseModel, Field


class ExportRequest(BaseModel):
    """Request to export data in a specific format."""

    format: str = "json"  # json, yaml, csv
    entities: list[str] = Field(default_factory=list)  # sessions, memories, agents, skills


class ExportStatus(BaseModel):
    """Status of an ongoing or completed export operation."""

    id: UUID
    status: str  # pending, in_progress, completed, failed
    progress: float = Field(default=0.0, ge=0.0, le=1.0)
    format: str = "json"
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
    completed_at: Optional[datetime] = None
    error: Optional[str] = None
    file_path: Optional[str] = None
