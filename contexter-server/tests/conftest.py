"""Shared conftest with fixtures for the contexter-server test suite."""

import pytest


@pytest.fixture
def any_uuid() -> str:
    """Return an arbitrary but valid UUID string for testing."""
    return "00000000-0000-0000-0000-000000000001"


@pytest.fixture
def sample_session_data() -> dict:
    """Return sample session data dict for testing."""
    return {
        "agent_id": "00000000-0000-0000-0000-000000000001",
        "project": "test-project",
        "name": "Test Session",
    }
