"""Edge case tests for service layer.

Covers:
- E-015: Concurrent session creates with same data → one 201, one 409
- E-024: 20 concurrent mocked bridge requests → all complete without error
- E-025: Slow bridge operation with asyncio.timeout → timeout exception
"""

import asyncio
from unittest.mock import AsyncMock

import pytest

from contexter_server.models.session import SessionCreate
from contexter_server.services.session_service import SessionService


@pytest.fixture
def mock_engine():
    """Create a mock StorageEngine."""
    return AsyncMock()


@pytest.fixture
def session_service(mock_engine):
    """Create a SessionService with mocked engine."""
    return SessionService(mock_engine)


class TestConcurrentSessionCreate:
    """E-015: Concurrent session creates with same data → one 201, one 409."""

    @pytest.mark.asyncio
    async def test_concurrent_create_with_same_id(self, mock_engine, any_uuid):
        """Two concurrent creates with same data — first succeeds, second fails."""
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
        service = SessionService(mock_engine)
        data = SessionCreate(agent_id=any_uuid, project="concurrent-test", name="Concurrent")

        results = await asyncio.gather(
            service.create(data),
            service.create(data),
            return_exceptions=True,
        )

        first, second = results
        assert first is not None
        assert str(first.id) == any_uuid
        assert second is None or isinstance(second, Exception)


class TestConcurrentBridgeRequests:
    """E-024: 20 concurrent mocked bridge requests → all complete without error."""

    @pytest.mark.asyncio
    async def test_twenty_concurrent_sessions(self, mock_engine, any_uuid):
        """Fire 20 concurrent session list requests — all complete."""
        mock_engine.list_sessions.return_value = [
            {
                "id": any_uuid.replace("000001", f"00000{i}"),
                "agent_id": any_uuid,
                "project": "test",
                "name": f"Session {i}",
                "status": "active",
                "started_at": "2026-07-25T10:00:00Z",
                "updated_at": "2026-07-25T10:00:00Z",
                "completed_at": None,
                "metadata": {},
            }
            for i in range(5)
        ]
        service = SessionService(mock_engine)

        coros = [service.list() for _ in range(20)]
        results = await asyncio.gather(*coros, return_exceptions=True)

        assert len(results) == 20
        exceptions = [r for r in results if isinstance(r, Exception)]
        assert len(exceptions) == 0
        assert all(len(r) == 5 for r in results if not isinstance(r, Exception))

    @pytest.mark.asyncio
    async def test_twenty_concurrent_bridge_calls_independent(self, mock_engine, any_uuid):
        """20 concurrent independent bridge calls — all complete."""
        service = SessionService(mock_engine)
        mock_engine.get_session.return_value = {
            "id": any_uuid,
            "agent_id": any_uuid,
            "project": "test",
            "name": "test",
            "status": "active",
            "started_at": "2026-07-25T10:00:00Z",
            "updated_at": "2026-07-25T10:00:00Z",
            "completed_at": None,
            "metadata": {},
        }

        coros = [service.get("test-id") for _ in range(20)]
        results = await asyncio.gather(*coros, return_exceptions=True)

        assert len(results) == 20
        exceptions = [r for r in results if isinstance(r, Exception)]
        assert len(exceptions) == 0


class TestBridgeTimeout:
    """E-025: Slow bridge operation with asyncio.timeout → timeout exception."""

    @pytest.mark.asyncio
    async def test_slow_bridge_operation_times_out(self, mock_engine, any_uuid):
        """A slow bridge operation exceeding the timeout should raise TimeoutError."""
        service = SessionService(mock_engine)

        async def slow_operation(*args, **kwargs):
            await asyncio.sleep(0.5)
            return {
                "id": any_uuid,
                "agent_id": any_uuid,
                "project": "test",
                "name": "slow",
                "status": "active",
                "started_at": "2026-07-25T10:00:00Z",
                "updated_at": "2026-07-25T10:00:00Z",
                "completed_at": None,
                "metadata": {},
            }

        mock_engine.get_session = AsyncMock(side_effect=slow_operation)

        with pytest.raises(TimeoutError):
            async with asyncio.timeout(0.1):
                await service.get("slow-id")

    @pytest.mark.asyncio
    async def test_fast_operation_succeeds_within_timeout(self, mock_engine, any_uuid):
        """A fast bridge operation should complete within the timeout."""
        mock_engine.get_session.return_value = {
            "id": any_uuid,
            "agent_id": any_uuid,
            "project": "test",
            "name": "fast",
            "status": "active",
            "started_at": "2026-07-25T10:00:00Z",
            "updated_at": "2026-07-25T10:00:00Z",
            "completed_at": None,
            "metadata": {},
        }
        service = SessionService(mock_engine)

        async with asyncio.timeout(0.1):
            result = await service.get("fast-id")
            assert result is not None
            assert result.name == "fast"
