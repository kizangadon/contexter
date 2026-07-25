"""Tests for SettingsService."""

from pathlib import Path
from unittest.mock import AsyncMock, patch

import pytest

from contexter_server.models.settings import (
    CacheConfig,
    LLMProviderConfig,
    Settings,
    StorageConfig,
)
from contexter_server.services.settings_service import SettingsService


@pytest.fixture
def mock_engine():
    engine = AsyncMock()
    return engine


@pytest.fixture
def service(mock_engine, tmp_path: Path) -> SettingsService:
    config_path = tmp_path / "config.yaml"
    return SettingsService(mock_engine, str(config_path))


class TestSettingsServiceLoad:
    """Tests for SettingsService.load."""

    @pytest.mark.asyncio
    async def test_creates_defaults_when_no_config(self, service, mock_engine):
        settings = await service.load()
        assert isinstance(settings, Settings)
        assert settings.storage.path == "~/.contexter/data"
        assert settings.cache.max_entries == 10000
        # Verify file was created
        assert service._config_path.exists()

    @pytest.mark.asyncio
    async def test_loads_existing_config(self, service, mock_engine):
        # Create a config file first
        initial = Settings(storage=StorageConfig(path="/custom/path"))
        await service.save(initial)

        # Re-read
        loaded = await service.load()
        assert loaded.storage.path == "/custom/path"
        assert loaded.cache.max_entries == 10000  # default from file

    @pytest.mark.asyncio
    async def test_returns_defaults_on_parse_error(self, service, mock_engine):
        service._config_path.parent.mkdir(parents=True, exist_ok=True)
        service._config_path.write_text("invalid: yaml: [")
        settings = await service.load()
        assert isinstance(settings, Settings)


class TestSettingsServiceSave:
    """Tests for SettingsService.save."""

    @pytest.mark.asyncio
    async def test_saves_settings(self, service, mock_engine):
        settings = Settings(
            storage=StorageConfig(path="/custom/path"),
            cache=CacheConfig(enabled=False, max_entries=500, ttl_secs=600),
        )
        await service.save(settings)
        assert service._config_path.exists()
        content = service._config_path.read_text()
        assert "/custom/path" in content
        assert "false" in content.lower() or "False" in content


class TestSettingsServiceGetSection:
    """Tests for SettingsService.get_section."""

    @pytest.mark.asyncio
    async def test_returns_section(self, service, mock_engine):
        settings = Settings(storage=StorageConfig(path="/test"))
        await service.save(settings)
        section = await service.get_section("storage")
        assert section is not None
        assert section["path"] == "/test"

    @pytest.mark.asyncio
    async def test_returns_none_for_unknown_section(self, service, mock_engine):
        section = await service.get_section("nonexistent")
        assert section is None

    @pytest.mark.asyncio
    async def test_returns_llm_providers_section(self, service, mock_engine):
        settings = Settings(llm_providers=[LLMProviderConfig(name="openai")])
        await service.save(settings)
        section = await service.get_section("llm_providers")
        assert section is not None
        assert len(section["items"]) == 1

    @pytest.mark.asyncio
    async def test_redacts_api_key_in_llm_providers(self, service, mock_engine):
        """api_key must be redacted when returned via get_section."""
        settings = Settings(
            llm_providers=[LLMProviderConfig(name="openai", api_key="sk-real-key-12345")]
        )
        await service.save(settings)
        section = await service.get_section("llm_providers")
        assert section is not None
        item = section["items"][0]
        assert item["api_key"] == "***redacted***", \
            "API key must be redacted in public response"

    @pytest.mark.asyncio
    async def test_internal_model_still_has_real_api_key(self, service, mock_engine):
        """Internal LLMProviderConfig must retain the real API key."""
        settings = Settings(
            llm_providers=[LLMProviderConfig(name="openai", api_key="sk-real-key-12345")]
        )
        await service.save(settings)
        loaded = await service.load()
        assert loaded.llm_providers[0].api_key == "sk-real-key-12345", \
            "Internal model must retain real API key"


class TestSettingsServiceUpdateSection:
    """Tests for SettingsService.update_section."""

    @pytest.mark.asyncio
    async def test_updates_section(self, service, mock_engine):
        await service.update_section("storage", {"path": "/new/path"})
        settings = await service.load()
        assert settings.storage.path == "/new/path"

    @pytest.mark.asyncio
    async def test_does_nothing_for_unknown_section(self, service, mock_engine):
        # Should not raise
        await service.update_section("nonexistent", {"key": "value"})


class TestSettingsServiceAsyncIO:
    """Tests that _load_yaml and _write_yaml offload I/O via asyncio.to_thread."""

    @pytest.mark.asyncio
    async def test_load_uses_to_thread(self, service, mock_engine):
        """load() should use asyncio.to_thread for file read via _load_yaml."""
        # Create a valid config file first
        initial = Settings(storage=StorageConfig(path="/custom"))
        await service.save(initial)

        with patch(
            "contexter_server.services.settings_service.asyncio.to_thread"
        ) as mock_tt:
            mock_tt.side_effect = lambda fn, *a, **kw: fn(*a)
            loaded = await service.load()
            assert loaded.storage.path == "/custom"
            # asyncio.to_thread was called at least once (for _load_yaml)
            assert mock_tt.called

    @pytest.mark.asyncio
    async def test_save_uses_to_thread(self, service, mock_engine):
        """save() should use asyncio.to_thread for file write via _write_yaml."""
        with patch(
            "contexter_server.services.settings_service.asyncio.to_thread"
        ) as mock_tt:
            mock_tt.side_effect = lambda fn, *a, **kw: fn(*a)
            settings = Settings(storage=StorageConfig(path="/custom"))
            await service.save(settings)
            # asyncio.to_thread was called at least once (for _write_yaml)
            assert mock_tt.called

    @pytest.mark.asyncio
    async def test_round_trip_preserves_section(self, service, mock_engine):
        """YAML round-trip preserves custom section values."""
        s = Settings(storage=StorageConfig(path="/custom/roundtrip"))
        await service.save(s)
        loaded = await service.load()
        assert loaded.storage.path == "/custom/roundtrip"

    def test_sync_write_yaml_exists(self, service):
        """_sync_write_yaml should be a callable method."""
        assert hasattr(service, "_sync_write_yaml")
        assert callable(service._sync_write_yaml)

    def test_sync_load_yaml_exists(self, service):
        """_sync_load_yaml should be a callable method."""
        assert hasattr(service, "_sync_load_yaml")
        assert callable(service._sync_load_yaml)
