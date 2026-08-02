"""Tests for the StorageEngine bridge (contexter_core.Engine wrapper)."""

import json
from unittest.mock import ANY, MagicMock, patch

import pytest

from contexter_server.core.bridge import (
    StorageEngine,
    _LARGE_CONTENT_THRESHOLD,
    _camelize_payload_keys,
    _snake_to_camel,
    _truncated_args_summary,
)


@pytest.fixture
def mock_engine():
    """Fixture that patches contexter_core.Engine and returns the mock instance."""
    with patch("contexter_server.core.bridge._SyncEngine") as mock:
        instance = MagicMock()
        mock.open.return_value = instance
        engine = StorageEngine(path="/tmp/test-contexter")
        yield engine, instance


class TestStorageEngineInit:
    """StorageEngine initialization tests."""

    def test_init_opens_engine(self):
        """StorageEngine should open the engine with given path."""
        with patch("contexter_server.core.bridge._SyncEngine") as mock:
            instance = MagicMock()
            mock.open.return_value = instance
            engine = StorageEngine(path="/tmp/test")
            mock.open.assert_called_once_with("/tmp/test")
            assert engine._max_workers == 8

    def test_init_custom_workers(self):
        """StorageEngine should accept custom max_workers."""
        with patch("contexter_server.core.bridge._SyncEngine") as mock:
            mock.open.return_value = MagicMock()
            engine = StorageEngine(path="/tmp/test", max_workers=8)
            assert engine._max_workers == 8

    def test_init_zero_workers_defaults(self):
        """StorageEngine should default to 8 workers if 0 passed (was 4)."""
        with patch("contexter_server.core.bridge._SyncEngine") as mock:
            mock.open.return_value = MagicMock()
            engine = StorageEngine(path="/tmp/test", max_workers=0)
            assert engine._max_workers == 8

    def test_init_default_max_workers_is_8(self):
        """Default max_workers should be 8 (was 4)."""
        with patch("contexter_server.core.bridge._SyncEngine") as mock:
            mock.open.return_value = MagicMock()
            engine = StorageEngine(path="/tmp/test")
            assert engine._max_workers == 8

    def test_init_env_var_override(self):
        """CONTEXTER_BRIDGE_POOL_SIZE env var should override default."""
        import os
        os.environ["CONTEXTER_BRIDGE_POOL_SIZE"] = "16"
        try:
            with patch("contexter_server.core.bridge._SyncEngine") as mock:
                mock.open.return_value = MagicMock()
                engine = StorageEngine(path="/tmp/test")
                assert engine._max_workers == 16
        finally:
            del os.environ["CONTEXTER_BRIDGE_POOL_SIZE"]

    def test_init_explicit_param_overrides_env_var(self):
        """Explicit max_workers should take precedence over env var."""
        import os
        os.environ["CONTEXTER_BRIDGE_POOL_SIZE"] = "2"
        try:
            with patch("contexter_server.core.bridge._SyncEngine") as mock:
                mock.open.return_value = MagicMock()
                engine = StorageEngine(path="/tmp/test", max_workers=12)
                assert engine._max_workers == 12
        finally:
            del os.environ["CONTEXTER_BRIDGE_POOL_SIZE"]

    def test_init_env_var_invalid_falls_back(self):
        """Invalid CONTEXTER_BRIDGE_POOL_SIZE should fall back to default."""
        import os
        os.environ["CONTEXTER_BRIDGE_POOL_SIZE"] = "not-a-number"
        try:
            with patch("contexter_server.core.bridge._SyncEngine") as mock:
                mock.open.return_value = MagicMock()
                engine = StorageEngine(path="/tmp/test")
                assert engine._max_workers == 8
        finally:
            del os.environ["CONTEXTER_BRIDGE_POOL_SIZE"]

    def test_init_env_var_zero_falls_back(self):
        """CONTEXTER_BRIDGE_POOL_SIZE=0 should fall back to default."""
        import os
        os.environ["CONTEXTER_BRIDGE_POOL_SIZE"] = "0"
        try:
            with patch("contexter_server.core.bridge._SyncEngine") as mock:
                mock.open.return_value = MagicMock()
                engine = StorageEngine(path="/tmp/test")
                assert engine._max_workers == 8
        finally:
            del os.environ["CONTEXTER_BRIDGE_POOL_SIZE"]

    def test_init_env_var_negative_falls_back(self):
        """Negative CONTEXTER_BRIDGE_POOL_SIZE should fall back to default."""
        import os
        os.environ["CONTEXTER_BRIDGE_POOL_SIZE"] = "-5"
        try:
            with patch("contexter_server.core.bridge._SyncEngine") as mock:
                mock.open.return_value = MagicMock()
                engine = StorageEngine(path="/tmp/test")
                assert engine._max_workers == 8
        finally:
            del os.environ["CONTEXTER_BRIDGE_POOL_SIZE"]

    def test_init_typo_env_var_ignored(self):
        """The misspelled legacy CONtexTER_* env var must be ignored (REQ-EV-002)."""
        import os
        os.environ["CONtexTER_BRIDGE_POOL_SIZE"] = "16"
        try:
            with patch("contexter_server.core.bridge._SyncEngine") as mock:
                mock.open.return_value = MagicMock()
                engine = StorageEngine(path="/tmp/test")
                assert engine._max_workers == 8
        finally:
            del os.environ["CONtexTER_BRIDGE_POOL_SIZE"]

    def test_init_canonical_env_var_wins_over_typo(self):
        """CONTEXTER_BRIDGE_POOL_SIZE takes precedence over the legacy typo (EC-EV-001)."""
        import os
        os.environ["CONTEXTER_BRIDGE_POOL_SIZE"] = "16"
        os.environ["CONtexTER_BRIDGE_POOL_SIZE"] = "2"
        try:
            with patch("contexter_server.core.bridge._SyncEngine") as mock:
                mock.open.return_value = MagicMock()
                engine = StorageEngine(path="/tmp/test")
                assert engine._max_workers == 16
        finally:
            del os.environ["CONTEXTER_BRIDGE_POOL_SIZE"]
            del os.environ["CONtexTER_BRIDGE_POOL_SIZE"]

    def test_os_expanduser_called(self):
        """os.path.expanduser should be called when a tilde path is provided."""
        with patch("contexter_server.core.bridge.os.path.expanduser") as mock_expand:
            mock_expand.return_value = "/home/user/.contexter"
            with patch("contexter_server.core.bridge._SyncEngine") as mock_engine:
                mock_engine.open.return_value = MagicMock()
                StorageEngine(path="~/.contexter")
                mock_expand.assert_called_once_with("~/.contexter")
                mock_engine.open.assert_called_once_with("/home/user/.contexter")


class TestStorageEngineSession:
    """Session CRUD tests."""

    @pytest.mark.asyncio
    async def test_create_session(self, mock_engine):
        engine, mock = mock_engine
        session_data = {"agent_id": "uuid-1", "project": "test"}
        result_json = json.dumps({"id": "sess-1", **session_data})
        mock.create_session.return_value = result_json

        result = await engine.create_session(session_data)
        assert result["id"] == "sess-1"
        # The bridge translates top-level keys to the engine's camelCase contract
        mock.create_session.assert_called_once_with(
            json.dumps({"agentId": "uuid-1", "project": "test"})
        )

    @pytest.mark.asyncio
    async def test_get_session_found(self, mock_engine):
        engine, mock = mock_engine
        mock.get_session.return_value = json.dumps({"id": "sess-1", "project": "test"})
        result = await engine.get_session("sess-1")
        assert result["id"] == "sess-1"
        mock.get_session.assert_called_once_with("sess-1")

    @pytest.mark.asyncio
    async def test_get_session_not_found(self, mock_engine):
        engine, mock = mock_engine
        mock.get_session.return_value = None
        result = await engine.get_session("nonexistent")
        assert result is None

    @pytest.mark.asyncio
    async def test_list_sessions(self, mock_engine):
        engine, mock = mock_engine
        sessions = [{"id": "s-1"}, {"id": "s-2"}]
        mock.list_sessions.return_value = json.dumps(sessions)
        result = await engine.list_sessions({"project": "test"}, limit=20, offset=5)
        assert len(result) == 2
        assert result[0]["id"] == "s-1"
        call_arg = json.loads(mock.list_sessions.call_args[0][0])
        assert call_arg["project"] == "test"
        assert call_arg["limit"] == 20
        assert call_arg["offset"] == 5

    @pytest.mark.asyncio
    async def test_list_sessions_no_filter(self, mock_engine):
        engine, mock = mock_engine
        mock.list_sessions.return_value = json.dumps([])
        result = await engine.list_sessions()
        assert result == []
        call_arg = json.loads(mock.list_sessions.call_args[0][0])
        assert call_arg["limit"] == 100
        assert call_arg["offset"] == 0

    @pytest.mark.asyncio
    async def test_list_sessions_with_pagination(self, mock_engine):
        """list_sessions should pass limit/offset in the filter dict."""
        engine, mock = mock_engine
        mock.list_sessions.return_value = json.dumps([])
        result = await engine.list_sessions({"project": "test"}, limit=50, offset=10)
        assert result == []
        call_arg = json.loads(mock.list_sessions.call_args[0][0])
        assert call_arg["limit"] == 50
        assert call_arg["offset"] == 10
        assert call_arg["project"] == "test"

    @pytest.mark.asyncio
    async def test_update_session(self, mock_engine):
        engine, mock = mock_engine
        mock.update_session.return_value = json.dumps({"id": "s-1", "status": "paused"})
        result = await engine.update_session("s-1", {"status": "paused"})
        assert result["status"] == "paused"
        mock.update_session.assert_called_once_with("s-1", json.dumps({"status": "paused"}))

    @pytest.mark.asyncio
    async def test_delete_session(self, mock_engine):
        engine, mock = mock_engine
        mock.delete_session.return_value = None
        await engine.delete_session("s-1")
        mock.delete_session.assert_called_once_with("s-1")

    @pytest.mark.asyncio
    async def test_count_sessions(self, mock_engine):
        engine, mock = mock_engine
        mock.count_sessions.return_value = 5
        result = await engine.count_sessions({"project": "test"})
        assert result == 5
        mock.count_sessions.assert_called_once_with(json.dumps({"project": "test"}))

    @pytest.mark.asyncio
    async def test_count_sessions_no_filter(self, mock_engine):
        engine, mock = mock_engine
        mock.count_sessions.return_value = 0
        result = await engine.count_sessions()
        assert result == 0
        mock.count_sessions.assert_called_once_with("{}")


class TestStorageEngineMemory:
    """Memory CRUD tests including large content path."""

    @pytest.mark.asyncio
    async def test_create_memory(self, mock_engine):
        engine, mock = mock_engine
        mem_data = {"session_id": "sid-1", "agent_id": "aid-1", "role": "user", "content": "Hello"}
        mock.create_memory.return_value = json.dumps({"id": "mem-1", **mem_data})
        result = await engine.create_memory(mem_data)
        assert result["id"] == "mem-1"
        # The bridge translates top-level keys to the engine's camelCase contract
        mock.create_memory.assert_called_once_with(
            json.dumps(
                {
                    "sessionId": "sid-1",
                    "agentId": "aid-1",
                    "role": "user",
                    "content": "Hello",
                }
            )
        )

    @pytest.mark.asyncio
    async def test_create_memory_large_content(self, mock_engine):
        """Memories with content >100KB should use create_memory_bytes."""
        engine, mock = mock_engine
        large_content = "x" * 102400  # 100KB
        mem_data = {
            "session_id": "sid-1",
            "agent_id": "aid-1",
            "role": "user",
            "content": large_content,
        }
        mock.create_memory_bytes.return_value = json.dumps({"id": "mem-1", "content": large_content})
        result = await engine.create_memory(mem_data)
        assert result["id"] == "mem-1"
        # Should have called create_memory_bytes with meta dict + bytes content
        mock.create_memory_bytes.assert_called_once()
        args = mock.create_memory_bytes.call_args[0]
        meta = json.loads(args[0])
        # The bridge translates top-level keys to the engine's camelCase contract
        assert meta["sessionId"] == "sid-1"
        # Rust NewMemory requires a content field; the bytes arg overwrites it,
        # so the bridge sends an empty placeholder instead of duplicating the
        # full content in the JSON payload (double-encode avoidance).
        assert meta["content"] == ""
        assert isinstance(args[1], bytes)
        assert len(args[1]) == 102400

    @pytest.mark.asyncio
    async def test_create_memory_large_content_exact_threshold(self, mock_engine):
        """Content exactly at threshold should still use standard path."""
        engine, mock = mock_engine
        boundary_content = "x" * 102399  # 99,999 bytes — just under 100KB
        mem_data = {
            "session_id": "sid-1",
            "agent_id": "aid-1",
            "role": "user",
            "content": boundary_content,
        }
        mock.create_memory.return_value = json.dumps({"id": "mem-1"})
        await engine.create_memory(mem_data)
        mock.create_memory.assert_called_once()

    @pytest.mark.asyncio
    async def test_get_memory(self, mock_engine):
        engine, mock = mock_engine
        mock.get_memory.return_value = json.dumps({"id": "mem-1", "content": "Hello"})
        result = await engine.get_memory("mem-1")
        assert result["content"] == "Hello"

    @pytest.mark.asyncio
    async def test_get_memory_not_found(self, mock_engine):
        engine, mock = mock_engine
        mock.get_memory.return_value = None
        result = await engine.get_memory("nonexistent")
        assert result is None

    @pytest.mark.asyncio
    async def test_search_memories(self, mock_engine):
        engine, mock = mock_engine
        results = [{"id": "mem-1", "score": 0.95}]
        mock.search_memories.return_value = json.dumps(results)
        result = await engine.search_memories({"query": "hello"}, limit=50, offset=10)
        assert len(result) == 1
        assert result[0]["score"] == 0.95
        # Verify limit/offset were included in the query dict
        call_arg = json.loads(mock.search_memories.call_args[0][0])
        assert call_arg["query"] == "hello"
        assert call_arg["limit"] == 50
        assert call_arg["offset"] == 10

    @pytest.mark.asyncio
    async def test_search_memories_default_pagination(self, mock_engine):
        """When limit/offset not provided, defaults should be 100/0."""
        engine, mock = mock_engine
        mock.search_memories.return_value = json.dumps([])
        result = await engine.search_memories({"query": "hello"})
        assert result == []
        call_arg = json.loads(mock.search_memories.call_args[0][0])
        assert call_arg["query"] == "hello"
        assert call_arg["limit"] == 100
        assert call_arg["offset"] == 0

    @pytest.mark.asyncio
    async def test_update_memory(self, mock_engine):
        engine, mock = mock_engine
        mock.update_memory.return_value = json.dumps({"id": "mem-1", "content": "Updated"})
        result = await engine.update_memory("mem-1", {"content": "Updated"})
        assert result["content"] == "Updated"

    @pytest.mark.asyncio
    async def test_update_memory_large_content(self, mock_engine):
        """Patch with content >100KB should use update_memory_bytes."""
        engine, mock = mock_engine
        large_content = "y" * 102400
        patch = {"content": large_content, "tokens": 100}
        mock.update_memory_bytes.return_value = json.dumps({"id": "mem-1", "content": large_content})
        result = await engine.update_memory("mem-1", patch)
        assert result is not None
        mock.update_memory_bytes.assert_called_once()
        args = mock.update_memory_bytes.call_args[0]
        assert args[0] == "mem-1"
        # Second positional arg is JSON-encoded meta dict without content
        meta = json.loads(args[1])
        assert "content" not in meta
        assert meta["tokens"] == 100
        # Third positional arg is the raw content bytes
        assert isinstance(args[2], bytes)
        assert len(args[2]) == 102400

    @pytest.mark.asyncio
    async def test_delete_memory(self, mock_engine):
        engine, mock = mock_engine
        mock.delete_memory.return_value = None
        await engine.delete_memory("mem-1")
        mock.delete_memory.assert_called_once_with("mem-1")

    @pytest.mark.asyncio
    async def test_count_memories(self, mock_engine):
        engine, mock = mock_engine
        mock.count_memories.return_value = 42
        result = await engine.count_memories({"session_id": "sid-1"})
        assert result == 42


class TestStorageEngineAgent:
    """Agent CRUD tests."""

    @pytest.mark.asyncio
    async def test_create_agent(self, mock_engine):
        engine, mock = mock_engine
        mock.create_agent.return_value = json.dumps({"id": "agent-1", "name": "TestAgent"})
        result = await engine.create_agent({"name": "TestAgent", "provider": "openai", "model": "gpt-4"})
        assert result["name"] == "TestAgent"

    @pytest.mark.asyncio
    async def test_get_agent(self, mock_engine):
        engine, mock = mock_engine
        mock.get_agent.return_value = json.dumps({"id": "agent-1"})
        result = await engine.get_agent("agent-1")
        assert result["id"] == "agent-1"

    @pytest.mark.asyncio
    async def test_get_agent_not_found(self, mock_engine):
        engine, mock = mock_engine
        mock.get_agent.return_value = None
        result = await engine.get_agent("nonexistent")
        assert result is None

    @pytest.mark.asyncio
    async def test_list_agents(self, mock_engine):
        engine, mock = mock_engine
        mock.list_agents.return_value = json.dumps([{"id": "a-1"}, {"id": "a-2"}])
        result = await engine.list_agents()
        assert len(result) == 2

    @pytest.mark.asyncio
    async def test_list_agents_with_pagination(self, mock_engine):
        """list_agents should pass limit/offset in the filter dict."""
        engine, mock = mock_engine
        mock.list_agents.return_value = json.dumps([])
        result = await engine.list_agents(limit=25, offset=5)
        assert result == []
        call_arg = json.loads(mock.list_agents.call_args[0][0])
        assert call_arg["limit"] == 25
        assert call_arg["offset"] == 5

    @pytest.mark.asyncio
    async def test_count_agents(self, mock_engine):
        engine, mock = mock_engine
        mock.count_agents.return_value = 3
        result = await engine.count_agents({"status": "active"})
        assert result == 3
        mock.count_agents.assert_called_once_with(json.dumps({"status": "active"}))

    @pytest.mark.asyncio
    async def test_count_agents_no_filter(self, mock_engine):
        engine, mock = mock_engine
        mock.count_agents.return_value = 0
        result = await engine.count_agents()
        assert result == 0
        mock.count_agents.assert_called_once_with("{}")

    @pytest.mark.asyncio
    async def test_update_agent(self, mock_engine):
        engine, mock = mock_engine
        mock.update_agent.return_value = json.dumps({"id": "agent-1", "temperature": 0.5})
        result = await engine.update_agent("agent-1", {"temperature": 0.5})
        assert result["temperature"] == 0.5

    @pytest.mark.asyncio
    async def test_delete_agent(self, mock_engine):
        engine, mock = mock_engine
        mock.delete_agent.return_value = None
        await engine.delete_agent("agent-1")
        mock.delete_agent.assert_called_once_with("agent-1")


class TestStorageEngineSkill:
    """Skill CRUD tests."""

    @pytest.mark.asyncio
    async def test_create_skill(self, mock_engine):
        engine, mock = mock_engine
        mock.create_skill.return_value = json.dumps({"id": "skill-1", "name": "search"})
        result = await engine.create_skill({"name": "search", "type": "search"})
        assert result["id"] == "skill-1"

    @pytest.mark.asyncio
    async def test_get_skill(self, mock_engine):
        engine, mock = mock_engine
        mock.get_skill.return_value = json.dumps({"id": "skill-1"})
        result = await engine.get_skill("skill-1")
        assert result["id"] == "skill-1"

    @pytest.mark.asyncio
    async def test_list_skills(self, mock_engine):
        engine, mock = mock_engine
        mock.list_skills.return_value = json.dumps([{"id": "s-1"}])
        result = await engine.list_skills()
        assert len(result) == 1

    @pytest.mark.asyncio
    async def test_list_skills_with_pagination(self, mock_engine):
        """list_skills should pass limit/offset in the filter dict."""
        engine, mock = mock_engine
        mock.list_skills.return_value = json.dumps([])
        result = await engine.list_skills(limit=10, offset=2)
        assert result == []
        call_arg = json.loads(mock.list_skills.call_args[0][0])
        assert call_arg["limit"] == 10
        assert call_arg["offset"] == 2

    @pytest.mark.asyncio
    async def test_count_skills(self, mock_engine):
        engine, mock = mock_engine
        mock.count_skills.return_value = 2
        result = await engine.count_skills({"category": "dev"})
        assert result == 2
        mock.count_skills.assert_called_once_with(json.dumps({"category": "dev"}))

    @pytest.mark.asyncio
    async def test_count_skills_no_filter(self, mock_engine):
        engine, mock = mock_engine
        mock.count_skills.return_value = 0
        result = await engine.count_skills()
        assert result == 0
        mock.count_skills.assert_called_once_with("{}")

    @pytest.mark.asyncio
    async def test_update_skill(self, mock_engine):
        engine, mock = mock_engine
        mock.update_skill.return_value = json.dumps({"id": "skill-1", "enabled": False})
        result = await engine.update_skill("skill-1", {"enabled": False})
        assert result["enabled"] is False

    @pytest.mark.asyncio
    async def test_delete_skill(self, mock_engine):
        engine, mock = mock_engine
        mock.delete_skill.return_value = None
        await engine.delete_skill("skill-1")
        mock.delete_skill.assert_called_once_with("skill-1")


class TestStorageEngineSettings:
    """Settings operations tests."""

    @pytest.mark.asyncio
    async def test_set_setting(self, mock_engine):
        engine, mock = mock_engine
        mock.set_setting.return_value = None
        await engine.set_setting("theme", "dark")
        mock.set_setting.assert_called_once_with("theme", "dark")

    @pytest.mark.asyncio
    async def test_get_setting(self, mock_engine):
        engine, mock = mock_engine
        mock.get_setting.return_value = "dark"
        result = await engine.get_setting("theme")
        assert result == "dark"

    @pytest.mark.asyncio
    async def test_get_setting_none(self, mock_engine):
        engine, mock = mock_engine
        mock.get_setting.return_value = None
        result = await engine.get_setting("nonexistent")
        assert result is None


class TestStorageEngineAudit:
    """Audit operations tests."""

    @pytest.mark.asyncio
    async def test_log_audit(self, mock_engine):
        engine, mock = mock_engine
        entry = {"entity_type": "session", "entity_id": "s-1", "action": "created"}
        mock.log_audit.return_value = None
        await engine.log_audit(entry)
        # The bridge translates top-level keys to the engine's camelCase contract
        mock.log_audit.assert_called_once_with(
            json.dumps({"entityType": "session", "entityId": "s-1", "action": "created"})
        )

    @pytest.mark.asyncio
    async def test_query_audit(self, mock_engine):
        engine, mock = mock_engine
        results = [{"id": "audit-1", "action": "created"}]
        mock.query_audit.return_value = json.dumps(results)
        result = await engine.query_audit({"entity_type": "session"})
        assert len(result) == 1
        assert result[0]["action"] == "created"


class TestStorageEngineMaintenance:
    """Maintenance operations tests."""

    @pytest.mark.asyncio
    async def test_flush(self, mock_engine):
        engine, mock = mock_engine
        mock.flush.return_value = None
        await engine.flush()
        mock.flush.assert_called_once()

    @pytest.mark.asyncio
    async def test_checkpoint(self, mock_engine):
        engine, mock = mock_engine
        mock.checkpoint.return_value = 42
        result = await engine.checkpoint()
        assert result == 42

    @pytest.mark.asyncio
    async def test_storage_size(self, mock_engine):
        engine, mock = mock_engine
        mock.storage_size.return_value = json.dumps({"bytes": 4096})
        result = await engine.storage_size()
        assert result["bytes"] == 4096

    @pytest.mark.asyncio
    async def test_status(self, mock_engine):
        engine, mock = mock_engine
        mock.status.return_value = json.dumps({"status": "ok"})
        result = await engine.status()
        assert result["status"] == "ok"

    @pytest.mark.asyncio
    async def test_clear_cache(self, mock_engine):
        engine, mock = mock_engine
        mock.clear_cache.return_value = None
        await engine.clear_cache()
        mock.clear_cache.assert_called_once()

    @pytest.mark.asyncio
    async def test_cache_telemetry(self, mock_engine):
        engine, mock = mock_engine
        mock.cache_telemetry.return_value = json.dumps({"entries": 100})
        result = await engine.cache_telemetry()
        assert result["entries"] == 100

    @pytest.mark.asyncio
    async def test_clear_cache_type(self, mock_engine):
        engine, mock = mock_engine
        mock.clear_cache_type.return_value = None
        await engine.clear_cache_type("session")
        mock.clear_cache_type.assert_called_once_with("session")


class TestStorageEngineErrors:
    """Error propagation tests."""

    @pytest.mark.asyncio
    async def test_invalid_method_raises(self, mock_engine):
        """Calling a non-existent method should raise AttributeError."""
        engine, mock = mock_engine
        with pytest.raises(AttributeError):
            await engine._run("nonexistent_method")

    @pytest.mark.asyncio
    async def test_create_session_engine_error(self, mock_engine, monkeypatch, tmp_path):
        """Engine errors should propagate as Python exceptions."""
        # Pin the diagnostics log away from the real ~/.contexter launch log.
        monkeypatch.setenv("CONTEXTER_LOG_FILE", str(tmp_path / "launch.log"))
        engine, mock = mock_engine
        mock.create_session.side_effect = RuntimeError("Engine failure")
        with pytest.raises(RuntimeError, match="Engine failure"):
            await engine.create_session({"project": "test"})


class TestStorageEngineImport:
    """Verify the import path is correct."""

    def test_import_from_contexter_core(self):
        """StorageEngine should import _SyncEngine from contexter_core."""

        # Verify the import source
        with patch("contexter_server.core.bridge._SyncEngine") as mock:
            instance = MagicMock()
            mock.open.return_value = instance
            engine = StorageEngine(path="/tmp/test")
            assert engine is not None


class TestStorageEngineLogging:
    """Logging behavior tests for StorageEngine._run."""

    @pytest.mark.asyncio
    async def test_run_logs_on_success(self, mock_engine):
        """_run should log one line on success with method, args, and duration."""
        engine, mock = mock_engine
        mock.create_session.return_value = json.dumps({"id": "sess-1"})

        with patch("contexter_server.core.bridge.logger") as mock_logger:
            await engine.create_session({"project": "test"})

            # Exactly one debug call — start+end combined; per-call logs are
            # DEBUG (REQ-PLB-001), not INFO
            mock_logger.debug.assert_called_once_with(
                "bridge_call_end",
                method="create_session",
                args_summary=ANY,
                duration_ms=ANY,
            )

    @pytest.mark.asyncio
    async def test_run_args_summary_truncated(self, mock_engine):
        """Args summary should be truncated at 200 chars."""
        engine, mock = mock_engine
        mock.create_memory.return_value = json.dumps({})
        long_str = "x" * 500
        mem = {"content": long_str}

        with patch("contexter_server.core.bridge.logger") as mock_logger:
            await engine.create_memory(mem)

            mock_logger.debug.assert_called_once()
            summary = mock_logger.debug.call_args.kwargs.get("args_summary", "")
            assert len(summary) <= 200

    @pytest.mark.asyncio
    async def test_run_logs_exception(self, mock_engine, monkeypatch, tmp_path):
        """_run should log a concise structured error (no traceback) before propagating."""
        # Pin the diagnostics log away from the real ~/.contexter launch log.
        monkeypatch.setenv("CONTEXTER_LOG_FILE", str(tmp_path / "launch.log"))
        engine, mock = mock_engine
        mock.create_session.side_effect = RuntimeError("Engine failure")

        with patch("contexter_server.core.bridge.logger") as mock_logger:
            with pytest.raises(RuntimeError, match="Engine failure"):
                await engine.create_session({"project": "test"})

            mock_logger.error.assert_called_once()
            args, kwargs = mock_logger.error.call_args
            assert "bridge_call_failed" in args
            assert kwargs.get("exc_info") is None, (
                "the concise error must never carry exc_info (traceback)"
            )
            assert kwargs.get("method") == "create_session"
            mock_logger.exception.assert_not_called()

    @pytest.mark.asyncio
    async def test_run_no_start_log(self, mock_engine):
        """_run should NOT log a start event — only the end event."""
        engine, mock = mock_engine
        mock.create_session.return_value = json.dumps({"id": "sess-1"})

        with patch("contexter_server.core.bridge.logger") as mock_logger:
            await engine.create_session({"project": "test"})

            # Ensure no bridge_call_start was logged (per-call end event is DEBUG)
            for call in mock_logger.debug.call_args_list:
                event = call.args[0] if call.args else None
                assert event != "bridge_call_start"


class TestTruncatedArgsSummary:
    """Tests for the _truncated_args_summary helper."""

    def test_empty_tuple(self):
        assert _truncated_args_summary(()) == "()"

    def test_single_string(self):
        result = _truncated_args_summary(("hello",))
        assert result == "('hello',)"

    def test_two_strings(self):
        result = _truncated_args_summary(("a", "b"))
        assert result == "('a', 'b')"

    def test_integer_arg(self):
        result = _truncated_args_summary((42,))
        assert result == "(42,)"

    def test_mixed_args(self):
        result = _truncated_args_summary(("id-1", '{"key": "val"}'))
        assert result == "('id-1', '{\"key\": \"val\"}')"

    def test_output_never_exceeds_max_len(self):
        """Output should never exceed max_len, even with large strings."""
        huge = "x" * 100_000
        result = _truncated_args_summary((huge,), max_len=200)
        assert len(result) <= 200

    def test_truncates_long_string(self):
        """Very long strings should be truncated in the summary."""
        long_str = "a" * 10_000
        result = _truncated_args_summary((long_str,), max_len=100)
        assert len(result) <= 100
        # Should contain an ellipsis indicating truncation
        assert "..." in result

    def test_truncates_long_bytes(self):
        """Very long bytes should be truncated in the summary."""
        long_bytes = b"b" * 10_000
        result = _truncated_args_summary((long_bytes,), max_len=100)
        assert len(result) <= 100
        assert "..." in result

    def test_multiple_long_strings(self):
        """Multiple long strings should all be truncated reasonably."""
        s1 = "x" * 5_000
        s2 = "y" * 5_000
        result = _truncated_args_summary((s1, s2), max_len=200)
        assert len(result) <= 200
        assert "..." in result

    def test_large_content_does_not_construct_full_string(self):
        """Ensure we don't construct the full repr of a huge string."""
        huge = "z" * 1_000_000  # 1 MB string
        # This should complete quickly (no 1MB allocation)
        result = _truncated_args_summary((huge,), max_len=50)
        assert len(result) <= 50

    def test_content_capped_at_documented_bound(self):
        """Content-bearing args are capped at the documented 64-char bound (REQ-BH-001)."""
        content = "x" * 10_000
        result = _truncated_args_summary((content,))
        assert len(result) <= 200
        assert "..." in result
        assert result.count("x") <= 64

    def test_full_content_never_appears(self):
        """The full content must never appear verbatim in the summary (REQ-BH-002)."""
        content = "a" * 10_000
        result = _truncated_args_summary((content,))
        assert content not in result

    def test_full_bytes_never_appears(self):
        """The full bytes content must never appear verbatim in the summary."""
        content = b"b" * 10_000
        result = _truncated_args_summary((content,))
        # The result is bounded (<=200 chars) so the 10KB payload cannot fit;
        # the bytes-level cap must still hold for the visible prefix.
        assert result.count("b") <= 64

    def test_exactly_at_cap_fully_logged(self):
        """Content exactly at the cap is logged in full without truncation marker (EC-BH-001)."""
        content = "c" * 64
        result = _truncated_args_summary((content,))
        assert content in result
        assert "..." not in result

    def test_empty_string_placeholder(self):
        """Empty string args render as a placeholder, not '' (EC-BH-003)."""
        result = _truncated_args_summary(("",))
        assert "'<empty>'" in result

    def test_empty_bytes_placeholder(self):
        """Empty bytes args render as a placeholder (EC-BH-003)."""
        result = _truncated_args_summary((b"",))
        assert "b'<empty>'" in result

    def test_multibyte_capped_by_chars(self):
        """Multibyte content is capped by characters, never split mid-codepoint (EC-BH-004)."""
        content = "\u4e2d" * 100  # 300 bytes
        result = _truncated_args_summary((content,))
        assert result.count("\u4e2d") <= 64
        assert "..." in result

    def test_secret_like_value_never_appears(self):
        """Long secret-like values must never appear verbatim (REQ-BH-003)."""
        secret = "sk-live-" + "a" * 100
        result = _truncated_args_summary((secret,))
        assert secret not in result


class TestStorageEngineThreadPool:
    """ThreadPoolExecutor routing tests."""

    @pytest.mark.asyncio
    async def test_run_uses_custom_pool(self, mock_engine):
        """_run should use self._pool via run_in_executor."""
        engine, mock = mock_engine
        mock.create_session.return_value = json.dumps({"id": "sess-1"})

        result = await engine.create_session({"project": "test"})
        assert result["id"] == "sess-1"
        assert engine._pool is not None
        assert not engine._pool._shutdown


class _CountingStr(str):
    """str subclass that counts encode() calls to prove single-encoding."""

    def __new__(cls, value: str) -> "_CountingStr":
        obj = str.__new__(cls, value)
        obj.encode_calls = 0
        return obj

    def encode(self, *args, **kwargs) -> bytes:
        self.encode_calls += 1
        return super().encode(*args, **kwargs)


class TestStorageEngineLargeContentByteLength:
    """Tests for byte-length-based large content detection (Bug 4)."""

    @pytest.mark.asyncio
    async def test_create_memory_bytes_path_encodes_content_once(self, mock_engine):
        """Bytes path must encode content exactly once (PF-02 double-encode fix)."""
        engine, mock = mock_engine
        content = _CountingStr("x" * _LARGE_CONTENT_THRESHOLD)
        mem_data = {
            "session_id": "sid-1",
            "agent_id": "aid-1",
            "role": "user",
            "content": content,
        }
        mock.create_memory_bytes.return_value = json.dumps({"id": "mem-1"})
        result = await engine.create_memory(mem_data)
        assert result["id"] == "mem-1"
        assert content.encode_calls == 1
        args = mock.create_memory_bytes.call_args[0]
        assert args[1] == b"x" * _LARGE_CONTENT_THRESHOLD

    @pytest.mark.asyncio
    async def test_update_memory_bytes_path_encodes_content_once(self, mock_engine):
        """update_memory bytes path must encode content exactly once (PF-02)."""
        engine, mock = mock_engine
        content = _CountingStr("y" * _LARGE_CONTENT_THRESHOLD)
        patch = {"content": content, "tokens": 100}
        mock.update_memory_bytes.return_value = json.dumps({"id": "mem-1"})
        result = await engine.update_memory("mem-1", patch)
        assert result is not None
        assert content.encode_calls == 1
        args = mock.update_memory_bytes.call_args[0]
        assert args[0] == "mem-1"
        assert args[2] == b"y" * _LARGE_CONTENT_THRESHOLD

    @pytest.mark.asyncio
    async def test_create_memory_ascii_at_threshold(self, mock_engine):
        """ASCII content at exactly 102400 bytes should use bytes path."""
        engine, mock = mock_engine
        content = "x" * _LARGE_CONTENT_THRESHOLD  # 102400 bytes
        mem_data = {
            "session_id": "sid-1",
            "agent_id": "aid-1",
            "role": "user",
            "content": content,
        }
        mock.create_memory_bytes.return_value = json.dumps({"id": "mem-1"})
        await engine.create_memory(mem_data)
        mock.create_memory_bytes.assert_called_once()
        mock.create_memory.assert_not_called()

    @pytest.mark.asyncio
    async def test_create_memory_ascii_just_under_threshold(self, mock_engine):
        """ASCII content just under 102400 bytes should use standard path."""
        engine, mock = mock_engine
        content = "x" * (_LARGE_CONTENT_THRESHOLD - 1)  # 102399 bytes
        mem_data = {
            "session_id": "sid-1",
            "agent_id": "aid-1",
            "role": "user",
            "content": content,
        }
        mock.create_memory.return_value = json.dumps({"id": "mem-1"})
        await engine.create_memory(mem_data)
        mock.create_memory.assert_called_once()
        mock.create_memory_bytes.assert_not_called()

    @pytest.mark.asyncio
    async def test_create_memory_multi_byte_triggers_bytes_path(self, mock_engine):
        """Multi-byte content: char count < threshold but byte count >= threshold."""
        engine, mock = mock_engine
        # CJK character is 3 bytes in UTF-8.  34134 × 3 = 102402 bytes >= threshold
        # 34134 chars is well below 102400 char threshold
        multi_byte_content = "\u4e2d" * 34134
        mem_data = {
            "session_id": "sid-1",
            "agent_id": "aid-1",
            "role": "user",
            "content": multi_byte_content,
        }
        mock.create_memory_bytes.return_value = json.dumps({"id": "mem-1"})
        await engine.create_memory(mem_data)
        mock.create_memory_bytes.assert_called_once()
        mock.create_memory.assert_not_called()

    @pytest.mark.asyncio
    async def test_create_memory_multi_byte_under_threshold(self, mock_engine):
        """Multi-byte content where both char count AND byte count are under threshold."""
        engine, mock = mock_engine
        # 30000 × 3 = 90000 bytes < 102400
        multi_byte_content = "\u4e2d" * 30000
        mem_data = {
            "session_id": "sid-1",
            "agent_id": "aid-1",
            "role": "user",
            "content": multi_byte_content,
        }
        mock.create_memory.return_value = json.dumps({"id": "mem-1"})
        await engine.create_memory(mem_data)
        mock.create_memory.assert_called_once()
        mock.create_memory_bytes.assert_not_called()

    @pytest.mark.asyncio
    async def test_update_memory_multi_byte_triggers_bytes_path(self, mock_engine):
        """update_memory with multi-byte content should use update_memory_bytes."""
        engine, mock = mock_engine
        multi_byte_content = "\u4e2d" * 34134  # 102402 bytes
        patch = {"content": multi_byte_content, "tokens": 100}
        mock.update_memory_bytes.return_value = json.dumps({"id": "mem-1"})
        result = await engine.update_memory("mem-1", patch)
        assert result is not None
        mock.update_memory_bytes.assert_called_once()
        mock.update_memory.assert_not_called()


class TestCamelizePayloadKeys:
    """Invariant tests for _snake_to_camel / _camelize_payload_keys (SEC-F04).

    Documented collision policy (REQ-CCI-002): camelization is a many-to-one
    mapping — ``foo_bar``, ``foo__bar`` and ``fooBar`` all map to ``fooBar``.
    When two input keys map to the same camelCase key, the LAST key in
    insertion order wins (Python dict-comprehension semantics); no error is
    raised. The output is fully deterministic for a given input dict, and no
    key that maps to a DISTINCT camelCase form is ever lost — data loss is
    confined to the documented last-wins collision policy.
    """

    def test_snake_to_camel_agent_id(self):
        """agent_id should become agentId."""
        assert _snake_to_camel("agent_id") == "agentId"

    def test_snake_to_camel_memory_type(self):
        """memory_type should become memoryType."""
        assert _snake_to_camel("memory_type") == "memoryType"

    def test_snake_to_camel_entity_type(self):
        """entity_type should become entityType."""
        assert _snake_to_camel("entity_type") == "entityType"

    def test_snake_to_camel_plain_word_unchanged(self):
        """Keys without underscores should pass through unchanged."""
        assert _snake_to_camel("project") == "project"

    def test_snake_to_camel_camelcase_identity(self):
        """Already-camelCase keys must be left untouched (no double transform)."""
        assert _snake_to_camel("agentId") == "agentId"
        assert _snake_to_camel("memoryType") == "memoryType"
        assert _snake_to_camel("fooBar") == "fooBar"

    def test_snake_to_camel_double_underscore_collapses(self):
        """foo__bar collapses to fooBar — collides with foo_bar and fooBar."""
        assert _snake_to_camel("foo__bar") == "fooBar"

    def test_snake_to_camel_leading_underscore_collides(self):
        """_foo maps to Foo — collides with an already-camel 'Foo' key."""
        assert _snake_to_camel("_foo") == "Foo"

    def test_snake_to_camel_trailing_underscore_collides(self):
        """foo_ maps to foo — collides with a plain 'foo' key."""
        assert _snake_to_camel("foo_") == "foo"

    def test_snake_to_camel_a_b_maps_to_aB_not_ab(self):
        """a_b maps to aB (not ab) — the a_b/ab pair is a trap, NOT a collision."""
        assert _snake_to_camel("a_b") == "aB"
        assert _snake_to_camel("ab") == "ab"

    def test_snake_to_camel_empty_string(self):
        """Empty string maps to empty string."""
        assert _snake_to_camel("") == ""

    def test_camelize_empty_payload(self):
        """Empty payload yields an empty dict."""
        assert _camelize_payload_keys({}) == {}

    def test_camelize_top_level_keys_only(self):
        """Nested dict values pass through untouched (opaque engine values)."""
        payload = {"meta_data": {"user_id": 1, "created_at": "t"}}
        result = _camelize_payload_keys(payload)
        assert result == {"metaData": {"user_id": 1, "created_at": "t"}}

    def test_camelize_non_string_keys_passthrough(self):
        """Non-string keys are never camelized."""
        assert _camelize_payload_keys({1: "one", None: "nil"}) == {1: "one", None: "nil"}

    def test_camelize_collision_last_wins_policy(self):
        """Documented policy: foo_bar and fooBar both map to fooBar; the LAST
        key in insertion order wins — no error is raised."""
        payload = {"foo_bar": "snake", "fooBar": "camel"}
        assert _camelize_payload_keys(payload) == {"fooBar": "camel"}

    def test_camelize_collision_reversed_insertion_order(self):
        """Reversing insertion order flips the winner — proves the policy is
        insertion-order last-wins, not value- or lexicographic-ordered."""
        payload = {"fooBar": "camel", "foo_bar": "snake"}
        assert _camelize_payload_keys(payload) == {"fooBar": "snake"}

    def test_camelize_double_underscore_collision(self):
        """foo_bar and foo__bar collide on fooBar; later key wins."""
        payload = {"foo_bar": 1, "foo__bar": 2}
        assert _camelize_payload_keys(payload) == {"fooBar": 2}

    def test_camelize_deterministic_same_input_same_output(self):
        """Adversarial input must yield identical output on every call."""
        payload = {"foo_bar": 1, "fooBar": 2, "a_b": 3, "aB": 4, "user_name": "don"}
        first = _camelize_payload_keys(payload)
        second = _camelize_payload_keys(payload)
        assert first == second
        assert list(first.items()) == list(second.items())

    def test_camelize_adversarial_set_no_loss_beyond_policy(self):
        """Adversarial set: colliding pairs resolve by last-wins; every key
        mapping to a DISTINCT camelCase form is preserved — no loss beyond the
        documented collision policy."""
        payload = {
            "a_b": 1,
            "ab": 2,  # NOT a collision: ab -> "ab", a_b -> "aB"
            "aB": 3,  # collides with a_b -> "aB"; last wins
            "foo_bar": 4,
            "foo__bar": 5,  # collides with foo_bar -> "fooBar"; last wins
            "user_name": "don",
        }
        assert _camelize_payload_keys(payload) == {
            "aB": 3,
            "ab": 2,
            "fooBar": 5,
            "userName": "don",
        }
