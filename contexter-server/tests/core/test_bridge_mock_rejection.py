"""Defense-in-depth: bridge dispatch must reject unittest.mock attribute types.

Even after the stub is gone, a mock object smuggled into the sync engine slot
(or a mock attribute on the engine class) must raise loudly instead of
propagating await-on-MagicMock corruption into the async services.
"""

import tempfile
from unittest.mock import MagicMock, patch

import pytest

import contexter_server.core.bridge as bridge_module
from contexter_server.core.bridge import StorageEngine


class _StubLikeEngine:
    """Mirrors the removed stub pattern: class attribute methods are MagicMocks."""

    create_session = MagicMock(name="Engine.create_session")
    count_agents = MagicMock(name="Engine.count_agents")
    count_skills = MagicMock(name="Engine.count_skills")

    @staticmethod
    def open(path: str) -> "_StubLikeEngine":
        return _StubLikeEngine()


class _StubLikeInstance:
    """A plain instance whose resolved attribute is a MagicMock."""

    create_session = MagicMock(name="Engine.create_session")


async def test_run_rejects_mock_class_attribute() -> None:
    """A Mock method on the sync engine CLASS must raise, not execute."""
    with tempfile.TemporaryDirectory() as tmp:
        engine = StorageEngine(path=tmp)
        with patch.object(bridge_module, "_SYNC_ENGINE_CLASS", _StubLikeEngine):
            with pytest.raises(TypeError, match=r"unittest\.mock"):
                await engine._run("create_session", "{}")


async def test_run_rejects_mock_class_attribute_count_agents() -> None:
    """EC-ACE-004: the mock guard applies to count_agents as well."""
    with tempfile.TemporaryDirectory() as tmp:
        engine = StorageEngine(path=tmp)
        with patch.object(bridge_module, "_SYNC_ENGINE_CLASS", _StubLikeEngine):
            with pytest.raises(TypeError, match=r"unittest\.mock"):
                await engine._run("count_agents", "{}")


async def test_run_rejects_mock_class_attribute_count_skills() -> None:
    """EC-ACE-004: the mock guard applies to count_skills as well."""
    with tempfile.TemporaryDirectory() as tmp:
        engine = StorageEngine(path=tmp)
        with patch.object(bridge_module, "_SYNC_ENGINE_CLASS", _StubLikeEngine):
            with pytest.raises(TypeError, match=r"unittest\.mock"):
                await engine._run("count_skills", "{}")


async def test_run_rejects_mock_instance_attribute() -> None:
    """A Mock method on the sync engine INSTANCE must raise, not execute."""
    with tempfile.TemporaryDirectory() as tmp:
        engine = StorageEngine(path=tmp)
        engine._engine = _StubLikeInstance()
        with pytest.raises(TypeError, match=r"unittest\.mock"):
            await engine._run("create_session", "{}")


async def test_run_executes_real_method_on_real_engine() -> None:
    """A genuine engine method must still execute normally."""
    with tempfile.TemporaryDirectory() as tmp:
        engine = StorageEngine(path=tmp)
        # _run is the raw seam — payloads must already speak the engine's
        # camelCase contract (the public wrappers camelize).
        raw = await engine._run(
            "create_session",
            '{"agentId": "00000000-0000-0000-0000-000000000001", '
            '"project": "bridge-probe"}',
        )
        assert '"id"' in raw
