"""Tests for NotificationService."""

import json
from datetime import datetime, timedelta, timezone
from unittest.mock import AsyncMock

import pytest

from contexter_server.models.notifications import Notification, NotificationList
from contexter_server.services.notification_service import NotificationService


@pytest.fixture
def mock_engine():
    engine = AsyncMock()
    return engine


@pytest.fixture
def service(mock_engine):
    return NotificationService(mock_engine)


class TestNotificationServiceList:
    """Tests for NotificationService.list."""

    @pytest.mark.asyncio
    async def test_returns_empty_list_when_no_notifications(self, service, mock_engine):
        mock_engine.get_setting.return_value = None
        result = await service.list()
        assert isinstance(result, NotificationList)
        assert result.notifications == []
        assert result.unread_count == 0

    @pytest.mark.asyncio
    async def test_returns_notifications_with_unread_count(self, service, mock_engine):
        mock_engine.get_setting.return_value = None
        mock_engine.set_setting.return_value = None
        service._add("Title 1", "Message 1", "info")
        service._add("Title 2", "Message 2", "warning")
        result = await service.list(limit=50)
        assert len(result.notifications) == 2
        assert result.unread_count == 2

    @pytest.mark.asyncio
    async def test_respects_limit(self, service, mock_engine):
        mock_engine.get_setting.return_value = None
        mock_engine.set_setting.return_value = None
        for i in range(5):
            service._add(f"Title {i}", f"Message {i}")
        result = await service.list(limit=3)
        assert len(result.notifications) == 3

    @pytest.mark.asyncio
    async def test_persists_via_bridge(self, service, mock_engine):
        """Adding a notification should persist to bridge on next async call."""
        mock_engine.get_setting.return_value = None
        mock_engine.set_setting.return_value = None

        service._add("Title", "Message")

        # Persist happens on the next async call (list triggers flush)
        result = await service.list()

        assert mock_engine.set_setting.await_count >= 1
        call_key = mock_engine.set_setting.await_args_list[0][0][0]
        assert call_key == "notifications"

    @pytest.mark.asyncio
    async def test_loads_from_bridge_on_init(self, service, mock_engine):
        """list should load notifications from bridge on first access."""
        notif = Notification(title="Existing", message="From bridge")
        notif_dict = notif.model_dump(mode="json")
        mock_engine.get_setting.return_value = json.dumps([notif_dict])
        mock_engine.set_setting.return_value = None

        # Create a fresh service (not using the fixture which may have cached data)
        fresh_service = NotificationService(mock_engine)
        result = await fresh_service.list()

        assert len(result.notifications) == 1
        assert result.notifications[0].title == "Existing"
        mock_engine.get_setting.assert_awaited_once_with("notifications")


class TestNotificationServiceMarkRead:
    """Tests for NotificationService.mark_read."""

    @pytest.mark.asyncio
    async def test_marks_notification_as_read(self, service, mock_engine):
        mock_engine.get_setting.return_value = None
        mock_engine.set_setting.return_value = None
        service._add("Title", "Message")
        n = list(service._notifications.values())[0]
        result = await service.mark_read(str(n.id))
        assert result is not None
        assert result.read is True

    @pytest.mark.asyncio
    async def test_returns_none_for_missing(self, service, mock_engine):
        mock_engine.get_setting.return_value = None
        result = await service.mark_read("nonexistent")
        assert result is None

    @pytest.mark.asyncio
    async def test_mark_read_persists_to_bridge(self, service, mock_engine):
        """Marking a notification as read should persist to bridge."""
        mock_engine.get_setting.return_value = None
        mock_engine.set_setting.return_value = None
        service._add("Title", "Message")
        n = list(service._notifications.values())[0]

        await service.mark_read(str(n.id))

        # Should have called set_setting with the updated notifications
        assert mock_engine.set_setting.await_count >= 1
        last_call_key = mock_engine.set_setting.await_args_list[-1][0][0]
        assert last_call_key == "notifications"


class TestNotificationServiceMarkAllRead:
    """Tests for NotificationService.mark_all_read."""

    @pytest.mark.asyncio
    async def test_marks_all_as_read(self, service, mock_engine):
        mock_engine.get_setting.return_value = None
        mock_engine.set_setting.return_value = None
        service._add("A", "Msg A")
        service._add("B", "Msg B")
        result = await service.mark_all_read()
        assert result.unread_count == 0
        assert len(result.notifications) == 2
        for n in result.notifications:
            assert n.read is True

    @pytest.mark.asyncio
    async def test_mark_all_read_persists_to_bridge(self, service, mock_engine):
        """Marking all as read should persist to bridge."""
        mock_engine.get_setting.return_value = None
        mock_engine.set_setting.return_value = None
        service._add("A", "Msg A")
        service._add("B", "Msg B")

        mock_engine.set_setting.reset_mock()
        await service.mark_all_read()

        assert mock_engine.set_setting.await_count >= 1
        call_key = mock_engine.set_setting.await_args_list[0][0][0]
        assert call_key == "notifications"


class TestNotificationServiceTTL:
    """Tests for TTL-based pruning of old notifications."""

    @pytest.mark.asyncio
    async def test_prunes_notifications_older_than_30_days_on_load(self, service, mock_engine):
        """Loading notifications should prune entries older than 30 days."""
        old_date = datetime.now(timezone.utc) - timedelta(days=31)
        recent_date = datetime.now(timezone.utc) - timedelta(days=1)

        old_notif = Notification(
            title="Old", message="Too old", created_at=old_date
        )
        recent_notif = Notification(
            title="Recent", message="Still relevant", created_at=recent_date
        )

        mock_engine.get_setting.return_value = json.dumps([
            old_notif.model_dump(mode="json"),
            recent_notif.model_dump(mode="json"),
        ])
        mock_engine.set_setting.return_value = None

        fresh_service = NotificationService(mock_engine)
        result = await fresh_service.list()

        # Only the recent notification should remain
        assert len(result.notifications) == 1
        assert result.notifications[0].title == "Recent"

    @pytest.mark.asyncio
    async def test_pruning_persists_cleaned_list_to_bridge(self, service, mock_engine):
        """After pruning old notifications, the cleaned list should be persisted."""
        old_date = datetime.now(timezone.utc) - timedelta(days=31)

        old_notif = Notification(
            title="Old", message="Too old", created_at=old_date
        )
        mock_engine.get_setting.return_value = json.dumps([
            old_notif.model_dump(mode="json"),
        ])
        mock_engine.set_setting.return_value = None

        fresh_service = NotificationService(mock_engine)
        await fresh_service.list()

        # Should have persisted the pruned (empty) list back
        set_calls = mock_engine.set_setting.await_args_list
        notification_saves = [
            c for c in set_calls if c[0][0] == "notifications"
        ]
        assert len(notification_saves) >= 1
        # The pruned list should be empty
        saved = json.loads(notification_saves[0][0][1])
        assert len(saved) == 0

    @pytest.mark.asyncio
    async def test_does_not_prune_recent_notifications(self, service, mock_engine):
        """Notifications within 30 days should be preserved."""
        recent_date = datetime.now(timezone.utc) - timedelta(days=29)
        notif1 = Notification(title="Recent 1", message="Msg 1", created_at=recent_date)
        notif2 = Notification(title="Recent 2", message="Msg 2")

        mock_engine.get_setting.return_value = json.dumps([
            notif1.model_dump(mode="json"),
            notif2.model_dump(mode="json"),
        ])
        mock_engine.set_setting.return_value = None

        fresh_service = NotificationService(mock_engine)
        result = await fresh_service.list()

        assert len(result.notifications) == 2
