"""Tests for ExportService."""

import json
from unittest.mock import AsyncMock

import pytest

from contexter_server.models.export import ExportRequest, ExportStatus
from contexter_server.services.export_service import ExportService


@pytest.fixture
def mock_engine():
    engine = AsyncMock()
    return engine


@pytest.fixture
def service(mock_engine):
    return ExportService(mock_engine)


class TestExportServiceSubmit:
    """Tests for ExportService.submit."""

    @pytest.mark.asyncio
    async def test_submits_export_with_entities(self, service, mock_engine):
        mock_engine.list_sessions.return_value = [{"id": "s1", "project": "test"}]
        mock_engine.search_memories.return_value = []
        mock_engine.list_agents.return_value = []
        mock_engine.list_skills.return_value = []

        request = ExportRequest(format="json", entities=["sessions"])
        result = await service.submit(request)
        assert isinstance(result, ExportStatus)
        assert result.status == "completed"
        assert result.progress == 1.0
        assert result.format == "json"
        mock_engine.list_sessions.assert_awaited_once()

    @pytest.mark.asyncio
    async def test_submits_export_all_entities_when_none_specified(self, service, mock_engine):
        mock_engine.list_sessions.return_value = []
        mock_engine.search_memories.return_value = []
        mock_engine.list_agents.return_value = []
        mock_engine.list_skills.return_value = []

        request = ExportRequest(format="json", entities=[])
        result = await service.submit(request)
        assert result.status == "completed"

    @pytest.mark.asyncio
    async def test_submits_yaml_format(self, service, mock_engine):
        mock_engine.list_sessions.return_value = []
        request = ExportRequest(format="yaml", entities=["sessions"])
        result = await service.submit(request)
        assert result.format == "yaml"

    @pytest.mark.asyncio
    async def test_submit_persists_to_bridge(self, service, mock_engine):
        """Submit must persist export status and data to bridge via set_setting."""
        mock_engine.list_sessions.return_value = []
        mock_engine.set_setting.return_value = None

        request = ExportRequest(format="json", entities=["sessions"])
        result = await service.submit(request)

        # Should have called set_setting with the export status key
        status_key = f"export_status:{result.id}"
        assert mock_engine.set_setting.await_count >= 2  # status + data
        # Verify at least one call stores the status
        status_calls = [
            call for call in mock_engine.set_setting.await_args_list
            if call[0][0] == status_key
        ]
        assert len(status_calls) >= 1, "Export status must be persisted to bridge"


class TestExportServiceGetStatus:
    """Tests for ExportService.get_status."""

    @pytest.mark.asyncio
    async def test_returns_status_for_existing_export(self, service, mock_engine):
        mock_engine.list_sessions.return_value = []
        request = ExportRequest(format="json", entities=["sessions"])
        created = await service.submit(request)
        status = await service.get_status(str(created.id))
        assert status is not None
        assert status.id == created.id

    @pytest.mark.asyncio
    async def test_returns_none_for_missing_export(self, service, mock_engine):
        mock_engine.get_setting.return_value = None
        result = await service.get_status("nonexistent")
        assert result is None

    @pytest.mark.asyncio
    async def test_get_status_falls_back_to_bridge_on_cache_miss(self, service, mock_engine):
        """get_status should fallback to bridge if not in cache."""
        mock_engine.list_sessions.return_value = []
        mock_engine.set_setting.return_value = None

        request = ExportRequest(format="json", entities=["sessions"])
        created = await service.submit(request)
        export_id_str = str(created.id)

        # Simulate cache miss by clearing the internal cache
        service._cache.clear()

        # Mock the bridge to return the persisted status
        raw = created.model_dump(mode="json", by_alias=True)
        mock_engine.get_setting.return_value = json.dumps(raw)

        status = await service.get_status(export_id_str)
        assert status is not None
        assert status.id == created.id
        mock_engine.get_setting.assert_awaited_once_with(f"export_status:{export_id_str}")


class TestExportServiceDownload:
    """Tests for ExportService.download."""

    @pytest.mark.asyncio
    async def test_downloads_exported_data(self, service, mock_engine):
        mock_engine.list_sessions.return_value = [{"id": "s1", "project": "test"}]
        request = ExportRequest(format="json", entities=["sessions"])
        created = await service.submit(request)
        data = await service.download(str(created.id))
        assert data is not None
        assert "sessions" in data

    @pytest.mark.asyncio
    async def test_returns_none_for_missing_download(self, service, mock_engine):
        mock_engine.get_setting.return_value = None
        result = await service.download("nonexistent")
        assert result is None

    @pytest.mark.asyncio
    async def test_download_persists_data_to_bridge(self, service, mock_engine):
        """Download should be available even from bridge after cache clear."""
        mock_engine.list_sessions.return_value = [{"id": "s1"}]
        mock_engine.set_setting.return_value = None

        request = ExportRequest(format="json", entities=["sessions"])
        created = await service.submit(request)
        export_id_str = str(created.id)

        # Clear cache to force bridge fallback
        service._cache.clear()
        service._data_cache.clear()

        persisted_data = json.dumps({"sessions": [{"id": "s1"}]})
        mock_engine.get_setting.return_value = persisted_data

        data = await service.download(export_id_str)
        assert data is not None
        assert "sessions" in data
        mock_engine.get_setting.assert_awaited_with(f"export_data:{export_id_str}")


class TestExportServiceLargeExports:
    """Tests that ExportService retrieves more than the 100-item default limit."""

    @pytest.mark.asyncio
    async def test_submit_retrieves_more_than_100_sessions(self, service, mock_engine):
        """Export must retrieve >100 items even though bridge defaults to limit=100."""
        many_sessions = [{"id": f"s{i}", "project": "test"} for i in range(150)]
        mock_engine.list_sessions.return_value = many_sessions
        mock_engine.search_memories.return_value = []
        mock_engine.list_agents.return_value = []
        mock_engine.list_skills.return_value = []
        mock_engine.set_setting.return_value = None

        request = ExportRequest(format="json", entities=["sessions"])
        result = await service.submit(request)

        data = await service.download(str(result.id))
        assert data is not None
        assert len(data["sessions"]) == 150

    @pytest.mark.asyncio
    async def test_submit_retrieves_more_than_100_all_entities(self, service, mock_engine):
        """Export with all entities must retrieve >100 items from each bridge call."""
        many_sessions = [{"id": f"s{i}"} for i in range(150)]
        many_memories = [{"id": f"m{i}"} for i in range(150)]
        many_agents = [{"id": f"a{i}"} for i in range(150)]
        many_skills = [{"id": f"sk{i}"} for i in range(150)]

        mock_engine.list_sessions.return_value = many_sessions
        mock_engine.search_memories.return_value = many_memories
        mock_engine.list_agents.return_value = many_agents
        mock_engine.list_skills.return_value = many_skills
        mock_engine.set_setting.return_value = None

        request = ExportRequest(format="json")
        result = await service.submit(request)

        data = await service.download(str(result.id))
        assert data is not None
        assert len(data["sessions"]) == 150
        assert len(data["memories"]) == 150
        assert len(data["agents"]) == 150
        assert len(data["skills"]) == 150

    @pytest.mark.asyncio
    async def test_submit_passes_explicit_limit_to_bridge(self, service, mock_engine):
        """ExportService must pass an explicit high limit to bridge methods."""
        mock_engine.list_sessions.return_value = []
        mock_engine.search_memories.return_value = []
        mock_engine.list_agents.return_value = []
        mock_engine.list_skills.return_value = []
        mock_engine.set_setting.return_value = None

        request = ExportRequest(format="json", entities=["sessions", "memories"])
        await service.submit(request)

        # Verify bridge was called with a limit >= 1000 (explicit high limit, not default)
        for call in mock_engine.list_sessions.await_args_list:
            assert "limit" not in call.kwargs or call.kwargs.get("limit", 0) >= 1000, \
                "Must pass explicit high limit to list_sessions"

    @pytest.mark.asyncio
    async def test_does_not_change_bridge_defaults(self, service, mock_engine):
        """The bridge method defaults must remain at limit=100 for other callers."""
        # Simply verify that bridge defaults are not modified — this test
        # validates that ExportService passes explicit limits, not that
        # bridge defaults changed.
        mock_engine.list_sessions.return_value = []
        mock_engine.search_memories.return_value = []
        mock_engine.list_agents.return_value = []
        mock_engine.list_skills.return_value = []
        mock_engine.set_setting.return_value = None

        request = ExportRequest(format="json", entities=["sessions", "memories", "agents", "skills"])
        await service.submit(request)

        # Verify all 4 bridge methods received limit as a keyword argument
        assert "limit" in mock_engine.list_sessions.await_args.kwargs
        assert "limit" in mock_engine.search_memories.await_args.kwargs
        assert "limit" in mock_engine.list_agents.await_args.kwargs
        assert "limit" in mock_engine.list_skills.await_args.kwargs


class TestExportServiceHistory:
    """Tests for ExportService.history."""

    @pytest.mark.asyncio
    async def test_returns_history(self, service, mock_engine):
        mock_engine.list_sessions.return_value = []
        await service.submit(ExportRequest(format="json", entities=["sessions"]))
        history = await service.history(limit=10)
        assert len(history) >= 1
        assert isinstance(history[0], ExportStatus)


class TestExportServiceLRUEviction:
    """Tests for LRU eviction in ExportService."""

    @pytest.mark.asyncio
    async def test_evicts_oldest_when_cache_exceeds_max(self, service, mock_engine):
        """Adding more than 100 exports should evict the oldest from cache."""
        mock_engine.search_memories.return_value = []
        mock_engine.list_sessions.return_value = []
        mock_engine.list_agents.return_value = []
        mock_engine.list_skills.return_value = []
        mock_engine.set_setting.return_value = None

        # Submit 101 exports (exceeds default maxsize of 100)
        ids = []
        for i in range(101):
            request = ExportRequest(format="json", entities=["sessions"])
            result = await service.submit(request)
            ids.append(str(result.id))

        # The first export should be evicted from cache
        first_id = ids[0]
        last_id = ids[-1]

        # First export should NOT be in cache (evicted)
        assert first_id not in service._cache

        # Last export should be in cache
        assert last_id in service._cache

    @pytest.mark.asyncio
    async def test_keeps_fresh_entries_in_cache(self, service, mock_engine):
        """Recent exports should remain in cache until eviction threshold."""
        mock_engine.search_memories.return_value = []
        mock_engine.list_sessions.return_value = []
        mock_engine.list_agents.return_value = []
        mock_engine.list_skills.return_value = []
        mock_engine.set_setting.return_value = None

        ids = []
        for i in range(99):
            request = ExportRequest(format="json", entities=["sessions"])
            result = await service.submit(request)
            ids.append(str(result.id))

        # All 99 should still be in cache
        for eid in ids:
            assert eid in service._cache, f"Export {eid} should be in cache"
