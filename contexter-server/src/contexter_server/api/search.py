"""FastAPI router for cross-entity search."""

from fastapi import APIRouter, Depends, Query

from contexter_server.models.search import SearchQuery, SearchResponse
from contexter_server.services.search_service import SearchService

from .deps import get_search_service

router = APIRouter(prefix="/api/v1/search", tags=["search"])


@router.get("", response_model=SearchResponse)
async def search(
    q: str = Query(..., description="Search query string"),
    type_filter: str | None = Query(None, alias="type", description="Filter result type"),
    project: str | None = Query(None, description="Filter by project"),
    page: int = Query(1, ge=1, description="Page number"),
    limit: int = Query(20, ge=1, le=100, description="Max results per page"),
    service: SearchService = Depends(get_search_service),
) -> SearchResponse:
    """Execute a cross-entity search across memories and sessions."""
    query = SearchQuery(query=q, type=type_filter, project=project, page=page, limit=limit)
    return await service.search(query)
