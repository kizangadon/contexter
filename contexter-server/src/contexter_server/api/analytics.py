"""FastAPI router for analytics, health, performance, resources, and costs."""

from fastapi import APIRouter, Depends

from contexter_server.models.analytics import (
    AnalyticsOverview,
    CostMetrics,
    ModelCost,
    PerformanceMetrics,
    ResourceUsage,
    ServiceStatus,
    SystemHealth,
)
from contexter_server.services.analytics_service import AnalyticsService

from .deps import get_analytics_service

router = APIRouter(prefix="/api/v1/analytics", tags=["analytics"])


@router.get("/overview", response_model=AnalyticsOverview)
async def overview(
    service: AnalyticsService = Depends(get_analytics_service),
) -> AnalyticsOverview:
    """Get a high-level system analytics overview."""
    return await service.get_overview()


@router.get("/health", response_model=SystemHealth)
async def health(
    service: AnalyticsService = Depends(get_analytics_service),
) -> SystemHealth:
    """Get system health status."""
    return await service.get_health()


@router.get("/performance", response_model=PerformanceMetrics)
async def performance(
    service: AnalyticsService = Depends(get_analytics_service),
) -> PerformanceMetrics:
    """Get system performance metrics."""
    return await service.get_performance()


@router.get("/resources", response_model=ResourceUsage)
async def resources(
    service: AnalyticsService = Depends(get_analytics_service),
) -> ResourceUsage:
    """Get current resource usage."""
    return await service.get_resources()


@router.get("/costs", response_model=CostMetrics)
async def costs_overview(
    service: AnalyticsService = Depends(get_analytics_service),
) -> CostMetrics:
    """Get aggregated cost metrics."""
    return await service.get_costs()


@router.get("/costs/models/{model_id}", response_model=ModelCost)
async def model_cost(
    model_id: str,
    service: AnalyticsService = Depends(get_analytics_service),
) -> ModelCost:
    """Get cost and usage for a specific model."""
    # TODO: implement model-specific cost tracking
    return ModelCost(model=model_id)


@router.get("/services", response_model=ServiceStatus)
async def service_status(
    service: AnalyticsService = Depends(get_analytics_service),
) -> ServiceStatus:
    """Get overall service status."""
    return await service.get_service_status()
