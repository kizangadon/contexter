"""Tests for session CLI formatting — None normalization."""

from unittest.mock import AsyncMock, MagicMock, patch

import pytest
from click.testing import CliRunner

from contexter_server.cli.main import cli
from contexter_server.cli.session_commands import _format_session


class MockSession:
    """Minimal mock session satisfying the interface used by _format_session."""

    def __init__(self, id: str, agent_id: str, project: str | None, name: str | None,
                 status: str = "active", started_at=None, updated_at=None, completed_at=None):
        self.id = id
        self.agent_id = agent_id
        self.project = project
        self.name = name
        self.status = status
        self.started_at = started_at
        self.updated_at = updated_at
        self.completed_at = completed_at


class TestFormatSession:
    """Tests for _format_session null-safe access."""

    def test_normalizes_none_project_to_empty_string(self):
        """_format_session should convert None project to empty string."""
        s = MockSession(
            id="id-1",
            agent_id="agent-1",
            project=None,
            name="Test",
            status="active",
        )
        result = _format_session(s)
        assert result["project"] == "" or result["project"] == ""

    def test_normalizes_none_name_to_empty_string(self):
        """_format_session should convert None name to empty string."""
        s = MockSession(
            id="id-1",
            agent_id="agent-1",
            project="test",
            name=None,
            status="active",
        )
        result = _format_session(s)
        assert result["name"] == ""

    def test_preserves_existing_values(self):
        """_format_session should keep non-None values unchanged."""
        s = MockSession(
            id="id-1",
            agent_id="agent-1",
            project="my-project",
            name="My Session",
            status="active",
        )
        result = _format_session(s)
        assert result["id"] == "id-1"
        assert result["agent_id"] == "agent-1"
        assert result["project"] == "my-project"
        assert result["name"] == "My Session"
        assert result["status"] == "active"
