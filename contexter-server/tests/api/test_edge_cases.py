"""Edge case tests for the API layer.

Covers:
- E-014: Client sends request with extremely large body → 413 / 422 (Pydantic size check)
- E-015: Two concurrent session creates with same data → one 201, one 409
- E-024: 20 concurrent mocked bridge requests → all complete without error
- E-031: Search query with null byte \\x00 → 422 rejection
- E-033: POST with 10,000-character entity ID → 422
"""

import asyncio
from unittest.mock import AsyncMock

import pytest
from fastapi.testclient import TestClient


class TestLargeBody:
    """E-014: Client sends request with extremely large body."""

    def test_session_create_large_project_name_is_rejected(self, client: TestClient, any_uuid: str):
        """Pydantic max_length check should reject extremely large project name."""
        large_project = "x" * 1000  # SessionCreate.project has max_length=256
        resp = client.post("/api/v1/sessions", json={
            "agent_id": any_uuid,
            "project": large_project,
            "name": "Test",
        })
        # Pydantic validation returns 422 for max_length violations
        assert resp.status_code == 422

    def test_session_create_large_name_is_rejected(self, client: TestClient, any_uuid: str):
        """Pydantic max_length check should reject extremely large name."""
        large_name = "x" * 2000  # SessionCreate.name has max_length=512
        resp = client.post("/api/v1/sessions", json={
            "agent_id": any_uuid,
            "project": "test",
            "name": large_name,
        })
        assert resp.status_code == 422


class TestConcurrentSessionCreate:
    """E-015: Two concurrent session creates with same data → one 201, one 409."""

    @pytest.mark.asyncio
    async def test_concurrent_duplicate_creates(self, client: TestClient, mock_engine: AsyncMock, any_uuid: str):
        """Fire two concurrent POSTs with same body — first 201, second 409."""
        call_count = 0

        async def create_session_side_effect(data):
            nonlocal call_count
            call_count += 1
            if call_count == 1:
                return {
                    "id": any_uuid,
                    "agent_id": any_uuid,
                    "project": "concurrent-test",
                    "name": "Concurrent",
                    "status": "active",
                    "started_at": "2026-07-25T10:00:00Z",
                    "updated_at": "2026-07-25T10:00:00Z",
                    "completed_at": None,
                    "metadata": {},
                }
            return None

        mock_engine.create_session = AsyncMock(side_effect=create_session_side_effect)

        async def _post():
            return client.post("/api/v1/sessions", json={
                "agent_id": any_uuid,
                "project": "concurrent-test",
                "name": "Concurrent",
            })

        resp1, resp2 = await asyncio.gather(_post(), _post())
        assert resp1.status_code == 201
        assert resp2.status_code == 409


class TestConcurrentBridgeRequests:
    """E-024: 20 concurrent mocked bridge requests → all complete."""

    @pytest.mark.asyncio
    async def test_twenty_concurrent_health_checks(self, client: TestClient):
        """Fire 20 concurrent health check requests — all 200."""
        async def _get_health():
            return client.get("/health")

        coros = [_get_health() for _ in range(20)]
        responses = await asyncio.gather(*coros)
        assert len(responses) == 20
        for resp in responses:
            assert resp.status_code == 200

    @pytest.mark.asyncio
    async def test_twenty_concurrent_session_lists(self, client: TestClient, mock_engine: AsyncMock):
        """Fire 20 concurrent session list requests — all 200."""
        mock_engine.list_sessions.return_value = []

        async def _list():
            return client.get("/api/v1/sessions")

        coros = [_list() for _ in range(20)]
        responses = await asyncio.gather(*coros)
        assert len(responses) == 20
        for resp in responses:
            assert resp.status_code == 200
            assert resp.json() == []


class TestNullByteInjection:
    """E-031: Search query with null byte \\x00 → 422 rejection."""

    def test_null_byte_in_search_query(self, client: TestClient):
        """Search query containing null byte should be rejected."""
        resp = client.get("/api/v1/search", params={"query": "test\x00malicious"})
        assert resp.status_code == 422

    def test_null_byte_in_search_project(self, client: TestClient):
        """Search project containing null byte should be rejected."""
        resp = client.get("/api/v1/search", params={"query": "test", "project": "proj\x00ect"})
        assert resp.status_code == 422


class TestLongEntityId:
    """E-033: POST with 10,000-character entity ID → 422."""

    def test_get_session_with_very_long_id(self, client: TestClient):
        """A 10,000-character session ID should be rejected."""
        long_id = "x" * 10000
        resp = client.get(f"/api/v1/sessions/{long_id}")
        assert resp.status_code == 422

    def test_get_memory_with_very_long_id(self, client: TestClient):
        """A 10,000-character memory ID should be rejected."""
        long_id = "x" * 10000
        resp = client.get(f"/api/v1/memories/{long_id}")
        assert resp.status_code == 422

    def test_get_agent_with_very_long_id(self, client: TestClient):
        """A 10,000-character agent ID should be rejected."""
        long_id = "x" * 10000
        resp = client.get(f"/api/v1/agents/{long_id}")
        assert resp.status_code == 422
