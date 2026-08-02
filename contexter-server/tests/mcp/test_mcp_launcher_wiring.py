"""Tests for ``run_mcp.py`` launcher wiring.

The MCP launcher MUST construct its six services on the ``StorageEngine``
bridge — the same async bridge the REST API uses — and MUST NOT hand the raw
engine (previously a MagicMock stub) to the services.

When the storage engine cannot be opened (RocksDB LOCK error, corrupt data,
unwritable data dir) the launcher MUST exit with a clean structured error on
stderr — never a raw Python/Rust traceback — persist full diagnostics to the
launch log, and exit with the documented nonzero code.
"""

import subprocess
import sys
import tempfile
import textwrap
import uuid
from pathlib import Path

import pytest

import run_mcp
from contexter_server.core.bridge import StorageEngine
from contexter_server.models.memory import MemoryCreate
from contexter_server.models.session import SessionCreate

# A helper subprocess that opens the engine and holds the RocksDB LOCK while
# it sleeps.  POSIX record locks only conflict between processes, so the lock
# must be held by a separate process — an in-process fcntl lock would silently
# be re-acquired by the second open.  The engine MUST be bound to a name:
# an unbound temporary would be dropped immediately, releasing the LOCK
# before the launcher ever opens the dir.
_HOLDER_SCRIPT = textwrap.dedent(
    """\
    import sys
    import time
    from contexter_core import Engine

    _engine = Engine.open(sys.argv[1])
    print("READY", flush=True)
    time.sleep(60)
    """
)


def test_build_services_returns_the_six_mcp_services() -> None:
    """``build_services`` must return exactly the six services MCP requires."""
    with tempfile.TemporaryDirectory() as tmp:
        services = run_mcp.build_services(tmp)
        assert set(services) == {
            "memory_service",
            "session_service",
            "agent_service",
            "skill_service",
            "analytics_service",
            "export_service",
        }


def test_all_services_are_wired_to_the_storage_engine_bridge() -> None:
    """Every service must hold a ``StorageEngine`` bridge, never a raw engine."""
    with tempfile.TemporaryDirectory() as tmp:
        services = run_mcp.build_services(tmp)
        for name, service in services.items():
            engine = service._engine
            assert isinstance(engine, StorageEngine), (
                f"{name} is wired to {type(engine).__name__!r}, expected "
                "StorageEngine bridge"
            )
            assert not isinstance(engine._engine, __import__("unittest").mock.Mock), (
                f"{name} holds a mock engine — the stub leaks into the live path"
            )


async def test_service_round_trip_via_launcher_wiring() -> None:
    """A session + memory created through launcher-built services must be real."""
    with tempfile.TemporaryDirectory() as tmp:
        services = run_mcp.build_services(tmp)
        agent_id = uuid.UUID("00000000-0000-0000-0000-000000000001")

        created = await services["session_service"].create(
            SessionCreate(agent_id=agent_id, project="mcp-live-fix-test")
        )
        assert created.id is not None
        assert created.status == "active"

        memory = await services["memory_service"].create(
            MemoryCreate(
                session_id=created.id,
                agent_id=agent_id,
                role="user",
                content="launcher wiring probe memory",
            )
        )
        assert memory.id is not None

        memories = await services["memory_service"].list()
        assert any(m.id == memory.id for m in memories), (
            "launcher-built services did not surface the created memory"
        )


def _spawn_engine_holder(data_dir) -> subprocess.Popen:
    """Open the engine in a child process and keep it alive (holds the RocksDB LOCK)."""
    holder = subprocess.Popen(
        [sys.executable, "-c", _HOLDER_SCRIPT, str(data_dir)],
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        text=True,
    )
    line = holder.stdout.readline()
    assert line.strip() == "READY", f"engine holder did not open the engine: {line!r}"
    return holder


def _corrupt_engine_data(data_dir) -> None:
    """Fabricate a RocksDB dir whose CURRENT/MANIFEST point at garbage."""
    (data_dir / "CURRENT").write_text("MANIFEST-999999\n")
    (data_dir / "MANIFEST-999999").write_text("garbage")
    (data_dir / "OPTIONS-000001").write_text("garbage")


def _assert_clean_launch_failure(exc_info, captured, log_file: Path, data_dir) -> None:
    """Shared contract: clean structured error, exit code, no traceback, log has detail."""
    assert exc_info.value.code == run_mcp.ENGINE_OPEN_EXIT_CODE, (
        "engine-open failure must exit with the documented nonzero code"
    )
    assert captured.out == "", "failure path must not write to stdout (MCP protocol)"
    assert "engine_open_failed" in captured.err, (
        "client-visible stderr must carry the structured error event"
    )
    assert str(data_dir) in captured.err, (
        "client-visible stderr must identify the engine path"
    )
    assert "Traceback" not in captured.err, (
        "raw traceback leaked to client-visible stderr"
    )
    log_text = log_file.read_text()
    assert "engine_open_failed" in log_text, "launch log must record the structured event"
    assert "Traceback" in log_text, "launch log must carry the full raw traceback"
    assert str(data_dir) in log_text, "launch log must identify the engine path"


def test_launch_failure_locked_dir_is_clean_and_logged(tmp_path, monkeypatch, capsys) -> None:
    """EC-LH-001 / AC-LH-001..002: RocksDB LOCK held by another process.

    The launcher must exit nonzero with a clean structured error (no raw
    traceback) and persist the full diagnostics to the launch log.
    """
    data_dir = tmp_path / "data"
    data_dir.mkdir()
    log_file = tmp_path / "launch.log"
    monkeypatch.setenv("CONTEXTER_PATH", str(data_dir))
    monkeypatch.setenv("CONTEXTER_LOG_FILE", str(log_file))

    holder = _spawn_engine_holder(data_dir)
    try:
        with pytest.raises(SystemExit) as exc_info:
            run_mcp.main()
    finally:
        holder.kill()
        holder.wait()

    _assert_clean_launch_failure(exc_info, capsys.readouterr(), log_file, data_dir)


def test_launch_failure_unwritable_data_dir_is_clean_and_logged(
    tmp_path, monkeypatch, capsys
) -> None:
    """EC-LH-002 / AC-LH-001..002: engine cannot create the data dir.

    A read-only parent makes ``create_dir_all`` fail with EACCES; the engine
    cannot repair the parent (it only chmods the data dir itself).
    """
    parent = tmp_path / "ro_parent"
    parent.mkdir()
    parent.chmod(0o500)
    data_dir = parent / "data"  # does not exist; cannot be created
    log_file = tmp_path / "launch.log"
    monkeypatch.setenv("CONTEXTER_PATH", str(data_dir))
    monkeypatch.setenv("CONTEXTER_LOG_FILE", str(log_file))
    try:
        with pytest.raises(SystemExit) as exc_info:
            run_mcp.main()
    finally:
        parent.chmod(0o700)  # keep pytest tmp cleanup possible

    _assert_clean_launch_failure(exc_info, capsys.readouterr(), log_file, data_dir)


def test_launch_failure_corrupt_engine_data_is_clean_and_logged(
    tmp_path, monkeypatch, capsys
) -> None:
    """EC-LH-003 / AC-LH-001..002: corrupt engine data fails cleanly with detail in logs."""
    data_dir = tmp_path / "data"
    data_dir.mkdir()
    _corrupt_engine_data(data_dir)
    log_file = tmp_path / "launch.log"
    monkeypatch.setenv("CONTEXTER_PATH", str(data_dir))
    monkeypatch.setenv("CONTEXTER_LOG_FILE", str(log_file))

    with pytest.raises(SystemExit) as exc_info:
        run_mcp.main()

    _assert_clean_launch_failure(exc_info, capsys.readouterr(), log_file, data_dir)


def test_build_services_still_raises_raw_on_engine_open_failure(tmp_path) -> None:
    """The raw-exception contract of ``build_services`` is unchanged.

    Only the launcher entry point converts engine-open failure into a clean
    error; direct callers keep receiving the original exception.
    """
    data_dir = tmp_path / "data"
    data_dir.mkdir()
    _corrupt_engine_data(data_dir)

    # Pinned to RuntimeError deliberately: corrupt engine data makes the PyO3
    # binding surface the Rust engine error as RuntimeError (verified live —
    # StorageEngine.__init__ -> Engine.open). A broad Exception match would
    # mask a future engine failure type, so the precise type is the contract.
    with pytest.raises(RuntimeError):
        run_mcp.build_services(str(data_dir))
