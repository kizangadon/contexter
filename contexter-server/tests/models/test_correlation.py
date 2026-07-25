"""Tests for correlation Pydantic models."""

from contexter_server.models.correlation import (
    CorrelationOverview,
    TimelineEntry,
    CorrelationTimeline,
    CorrelationCompare,
)


class TestCorrelationModels:
    """Correlation model validation tests."""

    def test_correlation_overview_defaults(self):
        """CorrelationOverview defaults."""
        o = CorrelationOverview()
        assert o.total_relationships == 0
        assert o.by_type == {}
        assert o.timeframe_hours == 24

    def test_correlation_overview_with_data(self):
        """CorrelationOverview with values."""
        o = CorrelationOverview(
            total_relationships=42,
            by_type={"session_memory": 20, "agent_skill": 22},
            timeframe_hours=48,
        )
        assert o.total_relationships == 42
        assert o.by_type["session_memory"] == 20
        assert o.timeframe_hours == 48

    def test_timeline_entry(self):
        """TimelineEntry with fields."""
        e = TimelineEntry(
            timestamp="2026-07-25T10:00:00Z",
            event_type="created",
            entity_id="abc-123",
            entity_type="session",
            description="Session created",
        )
        assert e.timestamp == "2026-07-25T10:00:00Z"
        assert e.description == "Session created"

    def test_correlation_timeline_defaults(self):
        """CorrelationTimeline defaults."""
        t = CorrelationTimeline()
        assert t.entries == []
        assert t.project is None
        assert t.agent_id is None

    def test_correlation_timeline_with_entries(self):
        """CorrelationTimeline with entries."""
        t = CorrelationTimeline(
            entries=[
                TimelineEntry(
                    timestamp="2026-07-25T10:00:00Z",
                    event_type="created",
                    entity_id="abc",
                    entity_type="memory",
                )
            ],
            project="test",
            agent_id="agent-1",
        )
        assert len(t.entries) == 1
        assert t.project == "test"
        assert t.agent_id == "agent-1"

    def test_correlation_compare_defaults(self):
        """CorrelationCompare defaults."""
        c = CorrelationCompare(entity_a_id="a", entity_b_id="b")
        assert c.entity_a_id == "a"
        assert c.entity_b_id == "b"
        assert c.shared_entities == 0
        assert c.relationship_strength == 0.0

    def test_correlation_compare_with_values(self):
        """CorrelationCompare with values."""
        c = CorrelationCompare(
            entity_a_id="a",
            entity_b_id="b",
            shared_entities=5,
            relationship_strength=0.75,
        )
        assert c.shared_entities == 5
        assert c.relationship_strength == 0.75
