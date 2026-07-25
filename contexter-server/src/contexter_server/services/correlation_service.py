"""Domain service for entity correlation analysis."""

import structlog

from contexter_server.core.bridge import StorageEngine

logger = structlog.get_logger(__name__)
from contexter_server.models.correlation import (
    CorrelationCompare,
    CorrelationOverview,
    CorrelationTimeline,
    TimelineEntry,
)


class CorrelationService:
    """Domain service for analysing relationships between entities."""

    def __init__(self, engine: StorageEngine) -> None:
        self._engine = engine

    async def get_overview(self, timeframe: str = "24h") -> CorrelationOverview:
        """Get an overview of entity correlations within a timeframe."""
        # TODO: implement real correlation analysis from bridge telemetry
        return CorrelationOverview(
            total_relationships=0,
            by_type={},
            timeframe_hours=24 if timeframe.endswith("h") else 168,
        )

    async def get_timeline(
        self,
        project: str | None = None,
        agent: str | None = None,
    ) -> CorrelationTimeline:
        """Get a timeline of correlated events, filtered by project or agent."""
        entries: list[TimelineEntry] = []

        # Query audit trail for events related to the filter criteria
        filter_dict: dict = {}
        if project:
            filter_dict["project"] = project
        if agent:
            filter_dict["actor"] = agent

        if filter_dict:
            try:
                audit_entries = await self._engine.query_audit(filter_dict)
                for ae in audit_entries:
                    entries.append(
                        TimelineEntry(
                            timestamp=ae.get("timestamp", ""),
                            event_type=ae.get("action", "unknown"),
                            entity_id=ae.get("entity_id", ""),
                            entity_type=ae.get("entity_type", "unknown"),
                            description=str(ae.get("details", {})),
                        )
                    )
            except Exception:
                logger.warning("audit_query_failed", exc_info=True)

        return CorrelationTimeline(
            entries=entries,
            project=project,
            agent_id=agent,
        )

    async def compare(self, a: str, b: str) -> CorrelationCompare:
        """Compare two entities and compute their relationship strength."""
        # TODO: implement real entity comparison from graph data
        return CorrelationCompare(
            entity_a_id=a,
            entity_b_id=b,
            shared_entities=0,
            relationship_strength=0.0,
        )
