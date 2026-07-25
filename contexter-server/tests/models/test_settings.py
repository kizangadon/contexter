"""Tests for settings Pydantic models."""

import pydantic
import pytest

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


class TestSettingsModels:
    """Settings model validation tests."""

    def test_project_config_defaults(self):
        """ProjectConfig defaults."""
        p = ProjectConfig()
        assert p.name == "default"
        assert p.description is None

    def test_storage_config_defaults(self):
        """StorageConfig defaults."""
        s = StorageConfig()
        assert s.path == "~/.contexter/data"
        assert s.autosave_interval_secs == 30

    def test_cache_config_defaults(self):
        """CacheConfig defaults."""
        c = CacheConfig()
        assert c.enabled is True
        assert c.max_entries == 10000
        assert c.ttl_secs == 300

    def test_mcp_server_config_defaults(self):
        """MCPServerConfig defaults to port 8052."""
        m = MCPServerConfig()
        assert m.host == "127.0.0.1"
        assert m.port == 8052

    def test_rest_config_defaults(self):
        """RESTConfig defaults to port 8051."""
        r = RESTConfig()
        assert r.host == "127.0.0.1"
        assert r.port == 8051

    def test_llm_provider_config(self):
        """LLMProviderConfig with fields."""
        lp = LLMProviderConfig(
            name="openai",
            api_key="sk-...",
            base_url="https://api.openai.com",
            default_model="gpt-4",
        )
        assert lp.name == "openai"
        assert lp.default_model == "gpt-4"

    def test_notifications_defaults(self):
        """NotificationsConfig defaults."""
        n = NotificationsConfig()
        assert n.enabled is True

    def test_versioning_config_defaults(self):
        """VersioningConfig defaults."""
        v = VersioningConfig()
        assert v.enabled is True
        assert v.max_versions_per_entity == 100

    def test_telemetry_config_defaults(self):
        """TelemetryConfig defaults."""
        t = TelemetryConfig()
        assert t.enabled is True
        assert t.sampling_rate == 1.0

    def test_analytics_config_defaults(self):
        """AnalyticsConfig defaults."""
        a = AnalyticsConfig()
        assert a.enabled is True
        assert a.retention_days == 90
        assert a.track_events == ["session", "memory", "search"]

    def test_analytics_config_retention_days_validation(self):
        """AnalyticsConfig retention_days must be 1-365."""
        with pytest.raises(pydantic.ValidationError):
            AnalyticsConfig(retention_days=0)
        with pytest.raises(pydantic.ValidationError):
            AnalyticsConfig(retention_days=366)
        a1 = AnalyticsConfig(retention_days=1)
        assert a1.retention_days == 1
        a2 = AnalyticsConfig(retention_days=365)
        assert a2.retention_days == 365

    def test_analytics_config_custom_values(self):
        """AnalyticsConfig with custom values."""
        a = AnalyticsConfig(
            enabled=False,
            retention_days=30,
            track_events=["session"],
        )
        assert a.enabled is False
        assert a.retention_days == 30
        assert a.track_events == ["session"]

    def test_settings_defaults(self):
        """Settings should compose all sub-configs."""
        s = Settings()
        assert s.project.name == "default"
        assert s.storage.path == "~/.contexter/data"
        assert s.mcp_server.port == 8052
        assert s.rest.port == 8051
        assert s.cache.enabled is True
        assert s.llm_providers == []
        assert s.notifications.enabled is True
        assert s.versioning.enabled is True
        assert s.telemetry.enabled is True
        assert s.analytics.enabled is True
        assert s.analytics.retention_days == 90
        assert s.analytics.track_events == ["session", "memory", "search"]

    def test_settings_with_custom_values(self):
        """Settings with custom configuration."""
        s = Settings(
            project=ProjectConfig(name="my-project", description="My project"),
            mcp_server=MCPServerConfig(port=9000),
            rest=RESTConfig(port=9001),
            llm_providers=[LLMProviderConfig(name="anthropic", default_model="claude-3")],
        )
        assert s.project.name == "my-project"
        assert s.mcp_server.port == 9000
        assert s.rest.port == 9001
        assert len(s.llm_providers) == 1
        assert s.llm_providers[0].name == "anthropic"

    def test_settings_with_analytics(self):
        """Settings with custom AnalyticsConfig."""
        s = Settings(
            analytics=AnalyticsConfig(
                enabled=False, retention_days=7, track_events=["search"]
            ),
        )
        assert s.analytics.enabled is False
        assert s.analytics.retention_days == 7
        assert s.analytics.track_events == ["search"]
