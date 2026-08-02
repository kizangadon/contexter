"""Launch-preamble log hygiene tests (REQ-SH-002, AC-SH-003, EC-SH-005).

Bug contract: 2026-08-01-success-path-log-hygiene.

The MCP launch path previously emitted a WARNING preamble whenever
``CONTEXTER_API_KEY`` was unset (open mode) — once at import time via the
module-level ``mcp = create_mcp_server()`` call, and again on every server
creation.  REQ-SH-002 requires that status be a DEBUG-level diagnostics
message, never a default-level (INFO+) stderr preamble (EC-SH-005), while
the signal must not be lost (EC-SH-001 analog).
"""

import importlib
import logging
import re

import pytest

import contexter_server.mcp_server as mcp_server_module
from contexter_server.mcp_server import create_mcp_server

_MCP_LOGGER = "contexter_server.mcp_server"

# structlog's ConsoleRenderer interleaves ANSI color codes into the rendered
# message; strip them before asserting on log content.
_ANSI_ESCAPE = re.compile(r"\x1b\[[0-9;]*m")


def _plain_message(record) -> str:
    return _ANSI_ESCAPE.sub("", record.getMessage())


class TestLaunchPreambleClean:
    """The open-mode launch must not emit WARNING+ records from mcp_server."""

    def test_create_server_without_api_key_no_warning(self, monkeypatch, caplog):
        """AC-SH-003: creating the server without an API key emits no WARNING."""
        monkeypatch.delenv("CONTEXTER_API_KEY", raising=False)

        with caplog.at_level(logging.INFO, logger=_MCP_LOGGER):
            mcp = create_mcp_server()

        assert mcp is not None
        warnings = [
            r
            for r in caplog.records
            if r.name == _MCP_LOGGER and r.levelno >= logging.WARNING
        ]
        assert warnings == [], (
            f"launch must emit zero WARNING+ records, got: "
            f"{[_plain_message(r) for r in warnings]}"
        )

    def test_module_import_without_api_key_no_warning(self, monkeypatch, caplog):
        """REQ-SH-002: importing the module (module-level create_mcp_server()
        call) must not emit a WARNING preamble."""
        monkeypatch.delenv("CONTEXTER_API_KEY", raising=False)

        with caplog.at_level(logging.INFO, logger=_MCP_LOGGER):
            importlib.reload(mcp_server_module)

        warnings = [
            r
            for r in caplog.records
            if r.name == _MCP_LOGGER and r.levelno >= logging.WARNING
        ]
        assert warnings == [], (
            f"module import must emit zero WARNING+ records, got: "
            f"{[_plain_message(r) for r in warnings]}"
        )

    def test_api_key_signal_preserved_at_debug(self, monkeypatch, caplog):
        """EC-SH-005: the unset-key status remains observable at DEBUG."""
        monkeypatch.delenv("CONTEXTER_API_KEY", raising=False)

        with caplog.at_level(logging.DEBUG, logger=_MCP_LOGGER):
            create_mcp_server()

        records = [
            r
            for r in caplog.records
            if r.name == _MCP_LOGGER
            and "CONTEXTER_API_KEY not set" in _plain_message(r)
        ]
        assert records, (
            "the unset-key status signal must still be logged (signal not lost)"
        )
        assert all(
            r.levelno == logging.DEBUG for r in records
        ), "unset-key status must be DEBUG, not WARNING (REQ-SH-002)"

    def test_api_key_configured_info_when_set(self, monkeypatch, caplog):
        """The key-configured branch stays an INFO lifecycle message."""
        monkeypatch.setenv("CONTEXTER_API_KEY", "test-key-123")

        with caplog.at_level(logging.INFO, logger=_MCP_LOGGER):
            mcp = create_mcp_server()

        assert mcp is not None
        infos = [
            r
            for r in caplog.records
            if r.name == _MCP_LOGGER
            and "mcp_server.api_key_configured" in _plain_message(r)
        ]
        assert infos, "key-configured status must be logged at INFO"
        assert all(
            r.levelno == logging.INFO for r in infos
        ), "key-configured status must remain INFO"
