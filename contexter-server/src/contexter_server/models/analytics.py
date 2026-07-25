"""Analytics domain models for system overview, health, and metrics."""

from pydantic import BaseModel, Field


class AnalyticsOverview(BaseModel):
    """High-level overview of system analytics."""

    total_sessions: int = 0
    total_memories: int = 0
    total_agents: int = 0
    total_skills: int = 0
    storage_size_bytes: int = 0
    uptime_seconds: int = 0


class SystemHealth(BaseModel):
    """System health status and resource usage."""

    status: str = "ok"  # ok, degraded, error
    uptime_seconds: int = 0
    memory_usage_mb: float = 0.0
    storage_size_bytes: int = 0
    cache_entries: int = 0


class PerformanceMetrics(BaseModel):
    """Performance metrics for the system."""

    avg_response_time_ms: float = 0.0
    total_operations: int = 0
    cache_hit_rate: float = 0.0


class ResourceUsage(BaseModel):
    """Current resource usage of the system."""

    cpu_percent: float = 0.0
    memory_mb: float = 0.0
    storage_mb: float = 0.0


class CostMetrics(BaseModel):
    """Cost metrics aggregated across models."""

    total_cost: float = 0.0
    cost_by_model: dict[str, float] = Field(default_factory=dict)


class ModelCost(BaseModel):
    """Cost and usage data for a specific model."""

    model: str
    cost: float = 0.0
    tokens: int = 0
    operations: int = 0


class ServiceStatus(BaseModel):
    """Status of an individual service."""

    name: str
    status: str = "ok"
    latency_ms: float = 0.0
