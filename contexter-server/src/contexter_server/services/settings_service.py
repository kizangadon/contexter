"""Domain service for system configuration and settings."""

import asyncio
import os
from pathlib import Path

import yaml

from contexter_server.core.bridge import StorageEngine
from contexter_server.models.settings import (
    AnalyticsConfig,
    CacheConfig,
    LLMProviderConfig,
    MCPServerConfig,
    NotificationsConfig,
    ProjectConfig,
    RESTConfig,
    Settings,
    StorageConfig,
    TelemetryConfig,
    VersioningConfig,
)

# Fields on LLMProviderConfig that contain secrets and MUST be redacted
# before returning to API consumers.
_SENSITIVE_PROVIDER_FIELDS = {"api_key"}


def _redact_sensitive_fields(item: dict) -> dict:
    """Return a copy of *item* with sensitive fields replaced by a sentinel.

    The original dict is not mutated. Only known-sensitive field names are
    touched — all other fields pass through unchanged.
    """
    result = dict(item)
    for field in _SENSITIVE_PROVIDER_FIELDS:
        if field in result and result[field] is not None:
            result[field] = "***redacted***"
    return result


def _default_settings() -> Settings:
    """Create a Settings instance with all default values."""
    return Settings()


class SettingsService:
    """Domain service for reading and writing system configuration.

    Operates on both the config YAML file on disk and individual settings
    stored in the bridge engine.
    """

    def __init__(
        self,
        engine: StorageEngine,
        config_path: str = "~/.contexter/config.yaml",
    ) -> None:
        self._engine = engine
        self._config_path = Path(config_path).expanduser()

    async def load(self) -> Settings:
        """Load settings from the config YAML file.

        If the file does not exist, create it with defaults and return those.
        """
        if not self._config_path.exists():
            settings = _default_settings()
            await self._write_yaml(settings)
            return settings

        try:
            raw = await self._load_yaml()
            return Settings.model_validate(raw)
        except Exception:
            return _default_settings()

    async def save(self, settings: Settings) -> None:
        """Save settings to the config YAML file."""
        await self._write_yaml(settings)

    async def get_section(self, section: str) -> dict | None:
        """Get a single configuration section by name."""
        settings = await self.load()
        section_map: dict[str, object] = {
            "project": settings.project,
            "storage": settings.storage,
            "cache": settings.cache,
            "mcp_server": settings.mcp_server,
            "rest": settings.rest,
            "llm_providers": settings.llm_providers,
            "notifications": settings.notifications,
            "versioning": settings.versioning,
            "telemetry": settings.telemetry,
            "analytics": settings.analytics,
        }
        val = section_map.get(section)
        if val is None:
            return None
        if isinstance(val, list):
            items = [v.model_dump() if hasattr(v, "model_dump") else v for v in val]
            if section == "llm_providers":
                items = [_redact_sensitive_fields(item) for item in items]
            return {"items": items}
        if hasattr(val, "model_dump"):
            return val.model_dump()
        return dict(val)  # type: ignore[arg-type]

    async def update_section(self, section: str, data: dict) -> None:
        """Update a single configuration section."""
        settings = await self.load()
        section_obj = getattr(settings, section, None)
        if section_obj is None:
            return
        if isinstance(section_obj, list):
            # For list sections like llm_providers, replace with parsed items
            items = data.get("items", data)
            parsed = [LLMProviderConfig.model_validate(item) for item in items]
            setattr(settings, section, parsed)
        else:
            for key, value in data.items():
                if hasattr(section_obj, key):
                    setattr(section_obj, key, value)
        await self._write_yaml(settings)

    async def _load_yaml(self) -> dict:
        """Load raw YAML content from the config file via thread pool."""
        raw: dict | None = await asyncio.to_thread(self._sync_load_yaml)
        return raw or {}

    def _sync_load_yaml(self) -> dict | None:
        """Synchronous YAML file read — runs in executor thread."""
        with self._config_path.open() as f:
            return yaml.safe_load(f)

    async def _write_yaml(self, settings: Settings) -> None:
        """Write settings to the config YAML file via thread pool."""
        self._config_path.parent.mkdir(parents=True, exist_ok=True)
        raw = settings.model_dump(mode="json")
        await asyncio.to_thread(self._sync_write_yaml, raw)

    def _sync_write_yaml(self, raw: dict) -> None:
        """Synchronous YAML file write — runs in executor thread."""
        with self._config_path.open("w") as f:
            yaml.dump(raw, f, default_flow_style=False)
