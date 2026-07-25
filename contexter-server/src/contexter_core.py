"""Stub for the contexter_core Rust PyO3 extension module.

In production, this module is the compiled Rust crate (contexter-core).
During development/testing, this stub provides the Engine class signature
so the Python bridge can be imported and tested via mocking.
"""

from unittest.mock import MagicMock

# All methods exposed by the Rust Engine via PyO3 #[pymethods].
# Defined at the class level so ``hasattr(Engine, name)`` works for
# both the real engine and the stub, and also so ``getattr`` on a
# patched MagicMock instance does not shadow method-existence checks.
_ENGINE_METHODS: list[str] = [
    # Session
    "create_session",
    "get_session",
    "list_sessions",
    "update_session",
    "delete_session",
    "count_sessions",
    # Memory
    "create_memory",
    "create_memory_bytes",
    "get_memory",
    "search_memories",
    "update_memory",
    "update_memory_bytes",
    "delete_memory",
    "count_memories",
    # Agent
    "create_agent",
    "get_agent",
    "list_agents",
    "update_agent",
    "delete_agent",
    # Skill
    "create_skill",
    "get_skill",
    "list_skills",
    "update_skill",
    "delete_skill",
    # Settings
    "set_setting",
    "get_setting",
    # Audit
    "log_audit",
    "query_audit",
    # Maintenance
    "flush",
    "checkpoint",
    "storage_size",
    "status",
    "clear_cache",
    "cache_telemetry",
    "clear_cache_type",
    # Internal
    "open",
]


def _make_mock_method(_name: str) -> MagicMock:
    """Return a MagicMock named after the Engine method it represents."""
    return MagicMock(name=f"Engine.{_name}")


class EngineMeta(type):
    """Metaclass that injects MagicMock stubs for every known method."""

    def __new__(mcs, name: str, bases: tuple, namespace: dict) -> type:
        for meth in _ENGINE_METHODS:
            if meth not in namespace:
                namespace[meth] = _make_mock_method(meth)
        return super().__new__(mcs, name, bases, namespace)


class Engine(metaclass=EngineMeta):
    """Stub matching the Rust Engine.open() interface.

    All public methods are MagicMock instances so they accept any call
    and can be patched by the test suite.
    """

    def __init__(self) -> None:
        pass

    @classmethod
    def open(cls, path: str) -> "Engine":
        _ = path
        return cls()
