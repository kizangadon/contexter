"""Tests asserting ``contexter_core`` resolves to the REAL Rust PyO3 extension.

These tests reproduce the root cause of the MCP live-failure bug: a committed
Python MagicMock stub (``src/contexter_core.py``) shadowed the real extension,
so every live engine call awaited a ``MagicMock``. They MUST fail while the
stub is present and pass once the real extension is installed and the stub is
removed.
"""

import json
import tempfile
from pathlib import Path
from unittest.mock import Mock


import contexter_core

# The full contract the stub mirrored — every method the real PyO3 Engine
# exposes via #[pymethods] (see contexter-core/src/bridge.rs).
_REAL_ENGINE_METHODS: list[str] = [
    "create_session",
    "get_session",
    "list_sessions",
    "update_session",
    "delete_session",
    "count_sessions",
    "create_memory",
    "create_memory_bytes",
    "get_memory",
    "search_memories",
    "update_memory",
    "update_memory_bytes",
    "delete_memory",
    "count_memories",
    "create_agent",
    "get_agent",
    "list_agents",
    "count_agents",
    "update_agent",
    "delete_agent",
    "create_skill",
    "get_skill",
    "list_skills",
    "count_skills",
    "update_skill",
    "delete_skill",
    "set_setting",
    "get_setting",
    "log_audit",
    "query_audit",
    "flush",
    "checkpoint",
    "storage_size",
    "status",
    "clear_cache",
    "cache_telemetry",
    "clear_cache_type",
    "open",
]

_AGENT_UUID = "00000000-0000-0000-0000-000000000001"


class TestRealExtensionResolution:
    """The import must resolve to the compiled extension, never the stub."""

    def test_module_is_compiled_extension_not_python_stub(self) -> None:
        """``contexter_core`` must resolve to the compiled extension, never the
        Python MagicMock stub that used to live in ``contexter-server/src``."""
        module_file = contexter_core.__file__ or ""
        assert "contexter-server/src" not in module_file, (
            f"contexter_core resolved into the server src tree: {module_file!r} — "
            "the MagicMock stub is shadowing the real Rust extension"
        )
        # The maturin wheel installs a package directory holding the abi3 .so
        # (e.g. site-packages/contexter_core/__init__.py + contexter_core.abi3.so).
        module_dir = Path(module_file).parent
        assert any(p.suffix == ".so" for p in module_dir.iterdir()), (
            f"no compiled extension found in {module_dir}"
        )

    def test_no_engine_method_is_a_mock(self) -> None:
        """No Engine method may resolve to a unittest.mock object."""
        engine_cls = contexter_core.Engine
        mock_methods = [
            name
            for name in _REAL_ENGINE_METHODS
            if isinstance(getattr(engine_cls, name, None), Mock)
        ]
        assert mock_methods == [], (
            f"Engine exposes unittest.mock attributes for: {mock_methods} — "
            "the stub is serving mocks in the live path"
        )

    def test_engine_class_has_no_mock_attributes_at_all(self) -> None:
        """Stronger check: no attribute of Engine at all is a Mock instance."""
        engine_cls = contexter_core.Engine
        mock_attrs = [
            name
            for name in dir(engine_cls)
            if isinstance(getattr(engine_cls, name, None), Mock)
        ]
        assert mock_attrs == [], f"Engine exposes mock attributes: {mock_attrs}"

    def test_engine_open_is_callable(self) -> None:
        """``Engine.open`` must be a real callable, not a MagicMock."""
        assert callable(contexter_core.Engine.open)


class TestRealEngineCrud:
    """Real engine round-trips against a temp store (never ~/.contexter)."""

    def test_open_returns_real_engine_instance(self) -> None:
        """``Engine.open`` on a temp dir must return a non-mock engine."""
        with tempfile.TemporaryDirectory() as tmp:
            engine = contexter_core.Engine.open(tmp)
            assert not isinstance(engine, Mock)
            assert callable(engine.create_session)

    def test_create_and_get_session_round_trip(self) -> None:
        """A session created via the real engine must be retrievable."""
        with tempfile.TemporaryDirectory() as tmp:
            engine = contexter_core.Engine.open(tmp)
            raw = engine.create_session(
                json.dumps(
                    {"agentId": _AGENT_UUID, "project": "mcp-live-fix-test"}
                )
            )
            session = json.loads(raw)
            assert session["id"], f"created session lacks id: {session!r}"

            fetched = json.loads(engine.get_session(session["id"]))
            assert fetched["id"] == session["id"]
            assert fetched["project"] == "mcp-live-fix-test"
            engine.flush()

    def test_create_and_search_memory_round_trip(self) -> None:
        """A memory created via the real engine must be searchable."""
        with tempfile.TemporaryDirectory() as tmp:
            engine = contexter_core.Engine.open(tmp)
            session_id = json.loads(
                engine.create_session(
                    json.dumps({"agentId": _AGENT_UUID, "project": "mcp-live-fix-test"})
                )
            )["id"]
            raw = engine.create_memory(
                json.dumps(
                    {
                        "sessionId": session_id,
                        "agentId": _AGENT_UUID,
                        "memoryType": "fact",
                        "content": "the quick brown fox jumps over the lazy dog",
                        "tags": ["probe"],
                    }
                )
            )
            memory = json.loads(raw)
            assert memory["id"], f"created memory lacks id: {memory!r}"

            results = json.loads(
                engine.search_memories(
                    json.dumps({"keywords": "fox", "limit": 100, "offset": 0})
                )
            )
            assert isinstance(results, list)
            assert any(m["id"] == memory["id"] for m in results), (
                f"search did not return created memory {memory['id']}: {results!r}"
            )
            engine.flush()

    def test_status_returns_real_health_payload(self) -> None:
        """``Engine.status()`` must return a real JSON health payload."""
        with tempfile.TemporaryDirectory() as tmp:
            engine = contexter_core.Engine.open(tmp)
            payload = json.loads(engine.status())
            assert payload["status"] == "ok"
