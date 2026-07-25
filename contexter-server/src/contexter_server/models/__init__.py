"""Pydantic domain models for the Contexter system.

All models use Pydantic v2 for validation and serialization.
Each module follows the ubiquitous language from the Contexter domain.
"""

from contexter_server.models.agent import Agent, AgentCreate, AgentPatch
from contexter_server.models.analytics import (
    AnalyticsOverview,
    CostMetrics,
    ModelCost,
    PerformanceMetrics,
    ResourceUsage,
    ServiceStatus,
    SystemHealth,
)
from contexter_server.models.audit import AuditEntry, AuditFilter
from contexter_server.models.correlation import (
    CorrelationCompare,
    CorrelationOverview,
    CorrelationTimeline,
    TimelineEntry,
)
from contexter_server.models.export import ExportRequest, ExportStatus
from contexter_server.models.memory import Memory, MemoryCreate, MemoryPatch
from contexter_server.models.notifications import Notification, NotificationList
from contexter_server.models.search import SearchQuery, SearchResult, SearchResponse
from contexter_server.models.session import Session, SessionCreate, SessionFilter, SessionPatch
from contexter_server.models.feedback import BugReport, FeatureSuggestion
from contexter_server.models.settings import (
    CacheConfig,
    LLMProviderConfig,
    MCPServerConfig,
    NotificationsConfig,
    ProjectConfig,
    RESTConfig,
    SectionUpdate,
    Settings,
    StorageConfig,
    TelemetryConfig,
    VersioningConfig,
)
from contexter_server.models.skill import Skill, SkillCreate, SkillPatch

__all__ = [
    "Session",
    "SessionCreate",
    "SessionPatch",
    "SessionFilter",
    "Memory",
    "MemoryCreate",
    "MemoryPatch",
    "Agent",
    "AgentCreate",
    "AgentPatch",
    "Skill",
    "SkillCreate",
    "SkillPatch",
    "AnalyticsOverview",
    "SystemHealth",
    "PerformanceMetrics",
    "ResourceUsage",
    "CostMetrics",
    "ModelCost",
    "ServiceStatus",
    "ProjectConfig",
    "StorageConfig",
    "CacheConfig",
    "MCPServerConfig",
    "RESTConfig",
    "LLMProviderConfig",
    "NotificationsConfig",
    "VersioningConfig",
    "TelemetryConfig",
    "Settings",
    "SectionUpdate",
    "BugReport",
    "FeatureSuggestion",
    "SearchQuery",
    "SearchResult",
    "SearchResponse",
    "ExportRequest",
    "ExportStatus",
    "CorrelationOverview",
    "TimelineEntry",
    "CorrelationTimeline",
    "CorrelationCompare",
    "AuditEntry",
    "AuditFilter",
    "Notification",
    "NotificationList",
]
