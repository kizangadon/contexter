"""Settings domain models for system configuration."""

from typing import Optional

from pydantic import BaseModel, Field


class ProjectConfig(BaseModel):
    """Project-level configuration."""

    name: str = "default"
    description: Optional[str] = None


class StorageConfig(BaseModel):
    """Storage configuration."""

    path: str = "~/.contexter/data"
    autosave_interval_secs: int = 30


class CacheConfig(BaseModel):
    """Cache configuration."""

    enabled: bool = True
    max_entries: int = 10000
    ttl_secs: int = 300


class MCPServerConfig(BaseModel):
    """MCP server network configuration."""

    host: str = "127.0.0.1"
    port: int = 8052


class RESTConfig(BaseModel):
    """REST API server network configuration."""

    host: str = "127.0.0.1"
    port: int = 8051


class LLMProviderConfig(BaseModel):
    """Configuration for an LLM provider."""

    name: str
    api_key: Optional[str] = None
    base_url: Optional[str] = None
    default_model: Optional[str] = None


class NotificationsConfig(BaseModel):
    """Notifications configuration."""

    enabled: bool = True


class VersioningConfig(BaseModel):
    """Versioning configuration for entity history."""

    enabled: bool = True
    max_versions_per_entity: int = 100


class TelemetryConfig(BaseModel):
    """Telemetry and observability configuration."""

    enabled: bool = True
    sampling_rate: float = 1.0


class AnalyticsConfig(BaseModel):
    """Analytics configuration for usage tracking and event collection."""

    enabled: bool = True
    retention_days: int = Field(default=90, ge=1, le=365)
    track_events: list[str] = Field(
        default_factory=lambda: ["session", "memory", "search"]
    )


class SectionUpdate(BaseModel):
    """Request model for updating a single configuration section.

    Wraps key-value pairs to set on the section. The ``values`` dict must
    contain at least one entry.
    """

    values: dict[str, object] = Field(..., min_length=1, description="Key-value pairs to update on the section")


class Settings(BaseModel):
    """Root settings model composing all sub-configurations."""

    project: ProjectConfig = Field(default_factory=ProjectConfig)
    storage: StorageConfig = Field(default_factory=StorageConfig)
    cache: CacheConfig = Field(default_factory=CacheConfig)
    mcp_server: MCPServerConfig = Field(default_factory=MCPServerConfig)
    rest: RESTConfig = Field(default_factory=RESTConfig)
    llm_providers: list[LLMProviderConfig] = Field(default_factory=list)
    notifications: NotificationsConfig = Field(default_factory=NotificationsConfig)
    versioning: VersioningConfig = Field(default_factory=VersioningConfig)
    telemetry: TelemetryConfig = Field(default_factory=TelemetryConfig)
    analytics: AnalyticsConfig = Field(default_factory=AnalyticsConfig)
