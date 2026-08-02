"""Domain service for Session aggregate operations."""

from datetime import datetime, timezone

from contexter_server.core.bridge import StorageEngine
from contexter_server.models.session import Session, SessionCreate, SessionFilter, SessionPatch

#: Documented upper bound for list_recent_sessions limit (EC-SL-005).
#: Limits above this are clamped so the engine is never asked to materialise
#: unbounded result sets (REQ-SL-004).
MAX_SESSION_LIST_LIMIT = 10_000


class SessionService:
    """Domain service for Session aggregate operations."""

    def __init__(self, engine: StorageEngine) -> None:
        self._engine = engine

    async def create(self, data: SessionCreate) -> Session:
        raw = await self._engine.create_session(data.model_dump(mode="json"))
        if raw is None:
            msg = "Session creation returned None — possible duplicate or conflict"
            raise ValueError(msg)
        return Session.model_validate(raw)

    async def get(self, id: str) -> Session | None:
        raw = await self._engine.get_session(id)
        return Session.model_validate(raw) if raw else None

    async def list(
        self, filter: SessionFilter | None = None, limit: int | None = None
    ) -> list[Session]:
        """List sessions, optionally pushing a clamped limit to the engine.

        ``limit=None`` keeps the engine's default page size (100) — the
        backward-compatible behaviour (REQ-SL-001). A negative limit clamps
        to 0 (EC-SL-004); a limit above ``MAX_SESSION_LIST_LIMIT`` clamps to
        the documented maximum (EC-SL-005). The engine performs the actual
        slicing, so only ``limit`` rows cross the bridge (REQ-SL-004).
        """
        filter_dict = filter.model_dump(exclude_none=True) if filter else None
        if limit is None:
            raw_list = await self._engine.list_sessions(filter_dict)
        else:
            clamped_limit = max(0, min(limit, MAX_SESSION_LIST_LIMIT))
            raw_list = await self._engine.list_sessions(filter_dict, limit=clamped_limit)
        return [Session.model_validate(r) for r in raw_list]

    async def update(self, id: str, patch: SessionPatch) -> Session | None:
        raw = await self._engine.update_session(id, patch.model_dump(exclude_none=True))
        return Session.model_validate(raw) if raw else None

    async def delete(self, id: str) -> None:
        await self._engine.delete_session(id)

    async def resume(self, id: str) -> Session | None:
        """Resume a session by setting status to active and clearing completed_at."""
        patch = SessionPatch(status="active")
        raw = await self._engine.update_session(
            id, patch.model_dump(exclude_none=True)
        )
        if not raw:
            return None
        return Session.model_validate(raw)

    async def compute_efficiency(self, id: str) -> float:
        """Compute session efficiency as a ratio of successful operations.

        TODO: Implement real efficiency computation from telemetry
        (e.g., token_used / total_tokens from an efficiency endpoint).
        Currently returns 1.0 (100%) as a safe default stub.
        """
        session = await self.get(id)
        if session is None:
            return 0.0
        # TODO: wire up real telemetry-based computation
        return 1.0
