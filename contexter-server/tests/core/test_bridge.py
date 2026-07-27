"""Tests for the StorageEngine bridge (contexter_core.Engine wrapper)."""

import json
from unittest.mock import ANY, MagicMock, patch

import pytest

from contexter_server.core.bridge import (
    StorageEngine,
    _LARGE_CONTENT_THRESHOLD,
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
        """CONtexTER_BRIDGE_POOL_SIZE env var should override default."""
        import os
        os.environ["CONtexTER_BRIDGE_POOL_SIZE"] = "16"
        try:
            with patch("contexter_server.core.bridge._SyncEngine") as mock:
                mock.open.return_value = MagicMock()
                engine = StorageEngine(path="/tmp/test")
                assert engine._max_workers == 16
        finally:
            del os.environ["CONtexTER_BRIDGE_POOL_SIZE"]

    def test_init_explicit_param_overrides_env_var(self):
        """Explicit max_workers should take precedence over env var."""
        import os
        os.environ["CONtexTER_BRIDGE_POOL_SIZE"] = "2"
        try:
            with patch("contexter_server.core.bridge._SyncEngine") as mock:
                mock.open.return_value = MagicMock()
                engine = StorageEngine(path="/tmp/test", max_workers=12)
                assert engine._max_workers == 12
        finally:
            del os.environ["CONtexTER_BRIDGE_POOL_SIZE"]

    def test_init_env_var_invalid_falls_back(self):
        """Invalid CONtexTER_BRIDGE_POOL_SIZE should fall back to default."""
        import os
        os.environ["CONtexTER_BRIDGE_POOL_SIZE"] = "not-a-number"
        try:
            with patch("contexter_server.core.bridge._SyncEngine") as mock:
                mock.open.return_value = MagicMock()
                engine = StorageEngine(path="/tmp/test")
                assert engine._max_workers == 8
        finally:
            del os.environ["CONtexTER_BRIDGE_POOL_SIZE"]

    def test_init_env_var_zero_falls_back(self):
        """CONtexTER_BRIDGE_POOL_SIZE=0 should fall back to default."""
        import os
        os.environ["CONtexTER_BRIDGE_POOL_SIZE"] = "0"
        try:
            with patch("contexter_server.core.bridge._SyncEngine") as mock:
                mock.open.return_value = MagicMock()
                engine = StorageEngine(path="/tmp/test")
                assert engine._max_workers == 8
        finally:
            del os.environ["CONtexTER_BRIDGE_POOL_SIZE"]

    def test_init_env_var_negative_falls_back(self):
        """Negative CONtexTER_BRIDGE_POOL_SIZE should fall back to default."""
        import os
        os.environ["CONtexTER_BRIDGE_POOL_SIZE"] = "-5"
        try:
            with patch("contexter_server.core.bridge._SyncEngine") as mock:
                mock.open.return_value = MagicMock()
                engine = StorageEngine(path="/tmp/test")
                assert engine._max_workers == 8
        finally:
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
        mock.create_session.assert_called_once_with(json.dumps(session_data))

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
        mock.create_memory.assert_called_once_with(json.dumps(mem_data))

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
        assert meta["session_id"] == "sid-1"
        assert "content" not in meta
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
        mock.log_audit.assert_called_once_with(json.dumps(entry))

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
    async def test_create_session_engine_error(self, mock_engine):
        """Engine errors should propagate as Python exceptions."""
        engine, mock = mock_engine
        mock.create_session.side_effect = RuntimeError("Engine failure")
        with pytest.raises(RuntimeError, match="Engine failure"):
            await engine.create_session({"project": "test"})


class TestStorageEngineImport:
    """Verify the import path is correct."""

    def test_import_from_contexter_core(self):
        """StorageEngine should import _SyncEngine from contexter_core."""
        import contexter_server.core.bridge as bridge_module

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

            # Exactly one info call — start+end combined
            mock_logger.info.assert_called_once_with(
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

            mock_logger.info.assert_called_once()
            summary = mock_logger.info.call_args.kwargs.get("args_summary", "")
            assert len(summary) <= 200

    @pytest.mark.asyncio
    async def test_run_logs_exception(self, mock_engine):
        """_run should log exception with logger.exception before propagating."""
        engine, mock = mock_engine
        mock.create_session.side_effect = RuntimeError("Engine failure")

        with patch("contexter_server.core.bridge.logger") as mock_logger:
            with pytest.raises(RuntimeError, match="Engine failure"):
                await engine.create_session({"project": "test"})

            mock_logger.exception.assert_called_once()
            args, _ = mock_logger.exception.call_args
            assert "bridge_call_failed" in args

    @pytest.mark.asyncio
    async def test_run_no_start_log(self, mock_engine):
        """_run should NOT log a start event — only the end event."""
        engine, mock = mock_engine
        mock.create_session.return_value = json.dumps({"id": "sess-1"})

        with patch("contexter_server.core.bridge.logger") as mock_logger:
            await engine.create_session({"project": "test"})

            # Ensure no bridge_call_start was logged
            for call in mock_logger.info.call_args_list:
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


class TestStorageEngineLargeContentByteLength:
    """Tests for byte-length-based large content detection (Bug 4)."""

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
