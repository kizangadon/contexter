"""Domain service for notification management."""

import json
from datetime import datetime, timedelta, timezone
from uuid import uuid4

from contexter_server.core.bridge import StorageEngine
from contexter_server.models.notifications import Notification, NotificationList

_NOTIFICATIONS_KEY = "notifications"
_TTL_DAYS = 30


class NotificationService:
    """Domain service for notification operations.

    Notifications are persisted to the StorageEngine bridge via
    ``set_setting``/``get_setting`` under a single key (``notifications``)
    as a JSON-serialised list.  An in-memory cache provides fast reads;
    every mutation is written through to the bridge.

    On load, any notification older than ``_TTL_DAYS`` (30) is pruned and
    the cleaned list is persisted back to the bridge.
    """

    def __init__(self, engine: StorageEngine) -> None:
        self._engine = engine
        self._notifications: dict[str, Notification] = {}
        self._loaded = False
        self._dirty = False

    # ------------------------------------------------------------------
    # Persistence helpers
    # ------------------------------------------------------------------

    async def _load(self) -> None:
        """Load notifications from bridge, pruning entries older than TTL."""
        if self._loaded:
            return
        self._loaded = True

        raw = await self._engine.get_setting(_NOTIFICATIONS_KEY)
        if raw is None:
            return

        raw_list = json.loads(raw)
        now = datetime.now(timezone.utc)
        cutoff = now - timedelta(days=_TTL_DAYS)

        pruned: list[Notification] = []
        for item in raw_list:
            notif = Notification.model_validate(item)
            if notif.created_at >= cutoff:
                pruned.append(notif)

        self._notifications = {str(n.id): n for n in pruned}

        # If any entries were pruned, persist the cleaned list.
        if len(pruned) < len(raw_list):
            await self._persist()

    async def _persist(self) -> None:
        """Write the current in-memory notifications list to the bridge."""
        serialised = [
            n.model_dump(mode="json") for n in self._notifications.values()
        ]
        await self._engine.set_setting(_NOTIFICATIONS_KEY, json.dumps(serialised))
        self._dirty = False

    async def _flush_if_dirty(self) -> None:
        """Persist to bridge if the in-memory state has changed."""
        if self._dirty:
            await self._persist()

    # ------------------------------------------------------------------
    # Public API
    # ------------------------------------------------------------------

    async def list(self, limit: int = 50) -> NotificationList:
        """List recent notifications with unread count."""
        await self._load()
        await self._flush_if_dirty()

        sorted_notifs = sorted(
            self._notifications.values(),
            key=lambda n: n.created_at,
            reverse=True,
        )
        unread = sum(1 for n in self._notifications.values() if not n.read)
        return NotificationList(
            notifications=sorted_notifs[:limit],
            unread_count=unread,
        )

    async def mark_read(self, id: str) -> Notification | None:
        """Mark a single notification as read."""
        await self._load()

        notif = self._notifications.get(id)
        if notif is None:
            return None
        notif.read = True
        await self._persist()
        return notif

    async def mark_all_read(self) -> NotificationList:
        """Mark all notifications as read."""
        await self._load()

        for notif in self._notifications.values():
            notif.read = True
        await self._persist()

        return NotificationList(
            notifications=list(self._notifications.values()),
            unread_count=0,
        )

    def _add(self, title: str, message: str, notification_type: str = "info") -> Notification:
        """Add a notification (internal helper for tests and system use).

        The notification is stored in-memory and marked dirty.  The next
        async call (``list``, ``mark_read``, ``mark_all_read``) will flush
        the change to the bridge.
        """
        notif = Notification(
            id=uuid4(),
            title=title,
            message=message,
            type=notification_type,
        )
        self._notifications[str(notif.id)] = notif
        self._dirty = True
        return notif
