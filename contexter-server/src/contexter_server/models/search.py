"""Search domain models for querying and results."""

from typing import Any, Optional
from uuid import UUID

from pydantic import BaseModel, Field


class SearchQuery(BaseModel):
    """Search query parameters."""

    query: str
    type: Optional[str] = None
    project: Optional[str] = None
    page: int = Field(default=1, ge=1)
    limit: int = Field(default=20, ge=1, le=100)


class SearchResult(BaseModel):
    """A single search result entry."""

    id: UUID
    type: str
    score: float = Field(default=0.0, ge=0.0, le=1.0)
    data: dict[str, Any] = Field(default_factory=dict)
    snippet: Optional[str] = None


class SearchResponse(BaseModel):
    """Paginated search response."""

    results: list[SearchResult] = Field(default_factory=list)
    total: int = 0
    page: int = 1
    limit: int = 20
