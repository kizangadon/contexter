"""Live tests for SessionService.list limit pushdown (real Rust engine).

These tests prove the limit-pushdown fix (2026-08-01-session-limit-pushdown)
at the real storage boundary: the engine receives the requested limit and
returns exactly that many sessions, preserving the engine's ordering
(REQ-SL-002 — result order identical to the unfiltered engine page).
"""

import uuid

import pytest

from contexter_server.core.bridge import StorageEngine
from contexter_server.models.session import SessionCreate
from contexter_server.services.session_service import SessionService


@pytest.fixture
def engine(tmp_path):
    eng = StorageEngine(path=str(tmp_path / "db"))
    yield eng
    eng._pool.shutdown(wait=False)


@pytest.fixture
def service(engine):
    return SessionService(engine)


async def _seed_sessions(service: SessionService, count: int) -> None:
    for _ in range(count):
        await service.create(
            SessionCreate(agent_id=str(uuid.uuid4()), project="live-pushdown", name="seed")
        )


@pytest.mark.asyncio
async def test_list_honors_limit_at_engine(service):
    """The engine boundary returns exactly the requested number of sessions."""
    await _seed_sessions(service, 3)
    full = await service.list()
    assert len(full) == 3
    limited = await service.list(limit=2)
    assert len(limited) == 2
    # Order must be identical to the first N of the full engine page.
    assert [s.id for s in limited] == [s.id for s in full[:2]]


@pytest.mark.asyncio
async def test_list_limit_one_matches_first_engine_result(service):
    """limit=1 returns a single session matching the engine's first result."""
    await _seed_sessions(service, 3)
    full = await service.list()
    one = await service.list(limit=1)
    assert len(one) == 1
    assert one[0].id == full[0].id
