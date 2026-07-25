"""Tests for ExportService — verify data and status use separate caches."""

from unittest.mock import AsyncMock

import pytest

from contexter_server.models.export import ExportRequest, ExportStatus
from contexter_server.services.export_service import ExportService


@pytest.fixture
def mock_engine():
    return AsyncMock()


@pytest.fixture
def service(mock_engine):
    return ExportService(mock_engine)


class TestExportServiceSeparateDataCache:
    """Verify export data is stored in a separate cache from status entries."""

    @pytest.mark.asyncio
    async def test_data_and_status_use_separate_caches(self, service, mock_engine):
        """Export data should be stored in _data_cache, status in _cache."""
        mock_engine.list_sessions.return_value = []
        request = ExportRequest(format="json", entities=["sessions"])
        created = await service.submit(request)

        export_id_str = str(created.id)
        # Status should be in _cache
        assert export_id_str in service._cache
        assert isinstance(service._cache[export_id_str], ExportStatus)
        # Data should be in _data_cache
        assert export_id_str in service._data_cache
        assert isinstance(service._data_cache[export_id_str], dict)

    @pytest.mark.asyncio
    async def test_download_returns_correct_data(self, service, mock_engine):
        """Download should return data from the separate _data_cache."""
        mock_engine.list_sessions.return_value = [{"id": "s1", "project": "test"}]
        request = ExportRequest(format="json", entities=["sessions"])
        created = await service.submit(request)

        data = await service.download(str(created.id))
        assert data is not None
        assert "sessions" in data

    @pytest.mark.asyncio
    async def test_history_only_includes_status_entries(self, service, mock_engine):
        """History should only include export status entries, not data."""
        mock_engine.list_sessions.return_value = []
        await service.submit(ExportRequest(format="json", entities=["sessions"]))
        history = await service.history(limit=10)
        for entry in history:
            assert isinstance(entry, ExportStatus)
