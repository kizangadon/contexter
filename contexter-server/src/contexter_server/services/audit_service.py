"""Domain service for audit trail operations."""

from datetime import datetime, timezone
from uuid import uuid4

from contexter_server.core.bridge import StorageEngine
from contexter_server.models.audit import AuditEntry, AuditFilter


class AuditService:
    """Domain service for audit trail management."""

    def __init__(self, engine: StorageEngine) -> None:
        self._engine = engine

    async def query(self, filter_data: AuditFilter) -> list[AuditEntry]:
        """Query audit entries matching the given filter."""
        raw_entries = await self._engine.query_audit(
            filter_data.model_dump(exclude_none=True)
        )
        return [AuditEntry.model_validate(r) for r in raw_entries]

    async def log(
        self,
        entity_type: str,
        entity_id: str,
        action: str,
        actor: str | None = None,
        details: dict | None = None,
    ) -> None:
        """Log an audit entry for a domain action."""
        entry = AuditEntry(
            id=uuid4(),
            entity_type=entity_type,
            entity_id=entity_id,
            action=action,
            actor=actor,
            timestamp=datetime.now(timezone.utc),
            details=details or {},
        )
        await self._engine.log_audit(entry.model_dump(mode="json"))
