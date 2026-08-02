"""Tests for the engine-failure stderr hygiene contract (bug EFS).

During MCP runtime, an engine-level failure in the bridge MUST NOT emit a
rich traceback to stderr.  stderr receives ONE concise structured line
(kind + bounded context, <512 chars per failure, no traceback); the full
exception diagnostics are persisted to the diagnostics log file (the MCP
launch log, ``CONTEXTER_LOG_FILE`` override); stdout stays pure.
"""

import logging
import sys
from pathlib import Path
from unittest.mock import MagicMock, patch

import pytest

import contexter_server.core.bridge as bridge_module
from contexter_server.core.bridge import StorageEngine


@pytest.fixture
def mock_engine():
    """Patch contexter_core.Engine and return (engine, mock instance)."""
    with patch("contexter_server.core.bridge._SyncEngine") as mock:
        instance = MagicMock()
        mock.open.return_value = instance
        engine = StorageEngine(path="/tmp/test-contexter")
        yield engine, instance


@pytest.fixture(autouse=True)
def _pin_diagnostics_log(monkeypatch, tmp_path):
    """Every test pins the diagnostics log to a temp file.

    The production default writes to ``~/.contexter/logs/mcp-launch.log``;
    tests must never touch the user's real launch log.
    """
    log_file = tmp_path / "logs" / "mcp-launch.log"
    monkeypatch.setenv("CONTEXTER_LOG_FILE", str(log_file))
    return log_file


def _attach_stderr_handler():
    """Route the bridge's stdlib logger to the real stderr.

    structlog's stdlib integration delivers records to the
    ``contexter_server.core.bridge`` stdlib logger; pytest intercepts
    records at the logging layer, so an explicit stderr handler makes the
    process-level stream observable via capfd (fd-level capture).
    """
    logger = logging.getLogger("contexter_server.core.bridge")
    handler = logging.StreamHandler(stream=sys.stderr)
    handler.setLevel(logging.ERROR)
    logger.addHandler(handler)
    return logger, handler


@pytest.mark.asyncio
async def test_engine_failure_stderr_bounded_no_traceback_stdout_pure(
    mock_engine, capfd
) -> None:
    """AC-EFS-001/003: engine failure -> one concise stderr line, no traceback, pure stdout."""
    engine, mock = mock_engine
    mock.create_session.side_effect = RuntimeError("Engine failure")

    logger, handler = _attach_stderr_handler()
    try:
        with pytest.raises(RuntimeError, match="Engine failure"):
            await engine.create_session({"project": "test"})
    finally:
        logger.removeHandler(handler)

    captured = capfd.readouterr()
    assert captured.out == "", "stdout must stay pure (MCP protocol)"
    assert "Traceback" not in captured.err, "rich traceback leaked to stderr"
    assert "bridge_call_failed" in captured.err, (
        "stderr must carry the structured failure kind"
    )
    assert len(captured.err) < 512, (
        f"stderr must be bounded (<512 chars per failure), got {len(captured.err)}"
    )


@pytest.mark.asyncio
async def test_engine_failure_full_diagnostics_in_log_file(
    mock_engine, _pin_diagnostics_log
) -> None:
    """AC-EFS-002: full exception detail is available in the diagnostics log file."""
    engine, mock = mock_engine
    mock.create_session.side_effect = RuntimeError("Engine failure")

    with pytest.raises(RuntimeError, match="Engine failure"):
        await engine.create_session({"project": "test"})

    log_text = _pin_diagnostics_log.read_text()
    assert "bridge_call_failed" in log_text, "log file must record the structured event"
    assert "RuntimeError" in log_text, "log file must carry the exception type"
    assert "Engine failure" in log_text, "log file must carry the exception message"
    assert "Traceback" in log_text, "log file must carry the full raw traceback"
    assert "create_session" in log_text, "log file must identify the failing method"


@pytest.mark.asyncio
async def test_engine_failure_log_record_has_no_exc_info(mock_engine, caplog) -> None:
    """The failure log record never carries exc_info -> no traceback can render."""
    engine, mock = mock_engine
    mock.create_session.side_effect = RuntimeError("Engine failure")

    with caplog.at_level(logging.ERROR, logger="contexter_server.core.bridge"):
        with pytest.raises(RuntimeError, match="Engine failure"):
            await engine.create_session({"project": "test"})

    records = [
        r
        for r in caplog.records
        if r.name == "contexter_server.core.bridge"
        and "bridge_call_failed" in r.getMessage()
    ]
    assert records, "expected a bridge_call_failed log record"
    record = records[-1]
    assert record.exc_info is None, "log record must not carry exception info"
    assert record.exc_text is None, "log record must not carry a formatted traceback"


@pytest.mark.asyncio
async def test_multiple_engine_failures_each_stderr_line_bounded(
    mock_engine, capfd
) -> None:
    """EC-EFS-001: each engine failure yields one independently bounded stderr line."""
    engine, mock = mock_engine
    mock.create_session.side_effect = RuntimeError("Engine failure")

    logger, handler = _attach_stderr_handler()
    try:
        with pytest.raises(RuntimeError, match="Engine failure"):
            await engine.create_session({"project": "test"})
        with pytest.raises(RuntimeError, match="Engine failure"):
            await engine.create_session({"project": "other"})
    finally:
        logger.removeHandler(handler)

    captured = capfd.readouterr()
    lines = [line for line in captured.err.splitlines() if line.strip()]
    assert len(lines) == 2, f"expected 2 concise stderr lines, got {lines!r}"
    assert "Traceback" not in captured.err
    for line in lines:
        assert len(line) < 512, f"each stderr line must stay bounded, got {len(line)}"


@pytest.mark.asyncio
async def test_unwritable_diagnostics_log_does_not_mask_engine_failure(
    mock_engine, monkeypatch, tmp_path
) -> None:
    """Best-effort diagnostics: an unwritable log path must not mask the engine error."""
    blocker = tmp_path / "blocker"
    blocker.write_text("i am a file, not a directory")
    monkeypatch.setenv("CONTEXTER_LOG_FILE", str(blocker / "logs" / "mcp-launch.log"))

    engine, mock = mock_engine
    mock.create_session.side_effect = RuntimeError("Engine failure")

    with pytest.raises(RuntimeError, match="Engine failure"):
        await engine.create_session({"project": "test"})


@pytest.mark.asyncio
async def test_diagnostics_writer_is_public_seam_for_the_launch_log(mock_engine) -> None:
    """The diagnostics writer resolves the launch log via CONTEXTER_LOG_FILE."""
    assert callable(bridge_module._write_runtime_failure_diagnostics)
    assert callable(bridge_module._resolve_diagnostics_log_path)
    resolved = bridge_module._resolve_diagnostics_log_path()
    assert isinstance(resolved, Path)
    assert resolved.name == "mcp-launch.log"
