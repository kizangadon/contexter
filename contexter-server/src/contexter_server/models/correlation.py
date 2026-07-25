"""Correlation domain models for entity relationship analysis."""

from typing import Optional

from pydantic import BaseModel, Field


class CorrelationOverview(BaseModel):
    """Overview of entity correlations."""

    total_relationships: int = 0
    by_type: dict[str, int] = Field(default_factory=dict)
    timeframe_hours: int = 24


class TimelineEntry(BaseModel):
    """A single event in a correlation timeline."""

    timestamp: str
    event_type: str
    entity_id: str
    entity_type: str
    description: Optional[str] = None


class CorrelationTimeline(BaseModel):
    """Timeline of correlated events."""

    entries: list[TimelineEntry] = Field(default_factory=list)
    project: Optional[str] = None
    agent_id: Optional[str] = None


class CorrelationCompare(BaseModel):
    """Comparison between two entities."""

    entity_a_id: str
    entity_b_id: str
    shared_entities: int = 0
    relationship_strength: float = 0.0
