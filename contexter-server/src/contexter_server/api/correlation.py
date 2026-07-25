"""FastAPI router for entity correlation analysis."""

from fastapi import APIRouter, Depends, Query

from contexter_server.models.correlation import (
    CorrelationCompare,
    CorrelationOverview,
    CorrelationTimeline,
)
from contexter_server.services.correlation_service import CorrelationService

from .deps import get_correlation_service

router = APIRouter(prefix="/api/v1/correlation", tags=["correlation"])


@router.get("/overview", response_model=CorrelationOverview)
async def correlation_overview(
    timeframe: str = Query("24h", description="Analysis timeframe (e.g. 24h, 7d)"),
    service: CorrelationService = Depends(get_correlation_service),
) -> CorrelationOverview:
    """Get an overview of entity correlations within a timeframe."""
    return await service.get_overview(timeframe=timeframe)


@router.get("/timeline", response_model=CorrelationTimeline)
async def correlation_timeline(
    project: str | None = Query(None, description="Filter by project"),
    agent: str | None = Query(None, alias="agent", description="Filter by agent ID"),
    service: CorrelationService = Depends(get_correlation_service),
) -> CorrelationTimeline:
    """Get a timeline of correlated events, filtered by project or agent."""
    return await service.get_timeline(project=project, agent=agent)


@router.get("/compare", response_model=CorrelationCompare)
async def correlation_compare(
    a: str = Query(..., description="First entity ID"),
    b: str = Query(..., description="Second entity ID"),
    service: CorrelationService = Depends(get_correlation_service),
) -> CorrelationCompare:
    """Compare two entities and compute their relationship strength."""
    return await service.compare(a=a, b=b)
