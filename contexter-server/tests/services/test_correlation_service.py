"""Tests for CorrelationService."""

from unittest.mock import AsyncMock, patch

import pytest

from contexter_server.models.correlation import CorrelationCompare, CorrelationOverview, CorrelationTimeline
from contexter_server.services.correlation_service import CorrelationService


@pytest.fixture
def mock_engine():
    engine = AsyncMock()
    return engine


@pytest.fixture
def service(mock_engine):
    return CorrelationService(mock_engine)


class TestCorrelationServiceGetOverview:
    """Tests for CorrelationService.get_overview."""

    @pytest.mark.asyncio
    async def test_returns_overview(self, service, mock_engine):
        result = await service.get_overview("24h")
        assert isinstance(result, CorrelationOverview)
        assert result.total_relationships == 0
        assert result.timeframe_hours == 24

    @pytest.mark.asyncio
    async def test_returns_overview_with_week_timeframe(self, service, mock_engine):
        result = await service.get_overview("7d")
        assert result.timeframe_hours == 168


class TestCorrelationServiceGetTimeline:
    """Tests for CorrelationService.get_timeline."""

    @pytest.mark.asyncio
    async def test_returns_empty_timeline_when_no_filters(self, service, mock_engine):
        result = await service.get_timeline()
        assert isinstance(result, CorrelationTimeline)
        assert result.entries == []
        assert result.project is None
        assert result.agent_id is None

    @pytest.mark.asyncio
    async def test_returns_timeline_with_project(self, service, mock_engine):
        mock_engine.query_audit.return_value = [
            {
                "timestamp": "2026-07-25T12:00:00Z",
                "action": "created",
                "entity_id": "sid-1",
                "entity_type": "session",
                "details": {"name": "test"},
            },
        ]
        result = await service.get_timeline(project="my-project")
        assert result.project == "my-project"
        assert len(result.entries) == 1
        assert result.entries[0].event_type == "created"

    @pytest.mark.asyncio
    async def test_handles_audit_query_error(self, service, mock_engine):
        mock_engine.query_audit.side_effect = Exception("bridge error")
        with patch("contexter_server.services.correlation_service.logger") as mock_logger:
            result = await service.get_timeline(project="my-project")
            assert len(result.entries) == 0
            mock_logger.warning.assert_called_once_with(
                "audit_query_failed",
                exc_info=True,
            )


class TestCorrelationServiceCompare:
    """Tests for CorrelationService.compare."""

    @pytest.mark.asyncio
    async def test_returns_compare_result(self, service, mock_engine):
        result = await service.compare("entity-a", "entity-b")
        assert isinstance(result, CorrelationCompare)
        assert result.entity_a_id == "entity-a"
        assert result.entity_b_id == "entity-b"
        assert result.relationship_strength == 0.0
