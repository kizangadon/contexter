"""Tests for notifications Pydantic models."""

import uuid
from datetime import datetime

from contexter_server.models.notifications import Notification, NotificationList


class TestNotificationModels:
    """Notification model validation tests."""

    def test_notification_defaults(self):
        """Notification should auto-generate id and timestamp."""
        n = Notification(title="Test", message="Hello")
        assert isinstance(n.id, uuid.UUID)
        assert isinstance(n.created_at, datetime)
        assert n.type == "info"
        assert n.read is False
        assert n.metadata == {}

    def test_notification_with_all_fields(self):
        """Notification with all fields."""
        n = Notification(
            title="Warning",
            message="Disk space low",
            type="warning",
            read=True,
            metadata={"severity": "high"},
        )
        assert n.title == "Warning"
        assert n.message == "Disk space low"
        assert n.type == "warning"
        assert n.read is True

    def test_notification_serialization(self):
        """Notification should serialize and deserialize."""
        n = Notification(title="Test", message="Body")
        data = n.model_dump()
        restored = Notification.model_validate(data)
        assert restored.title == "Test"
        assert restored.message == "Body"

    def test_notification_list_defaults(self):
        """NotificationList defaults."""
        nl = NotificationList()
        assert nl.notifications == []
        assert nl.unread_count == 0

    def test_notification_list_with_items(self):
        """NotificationList with items."""
        n1 = Notification(title="A", message="Msg 1")
        n2 = Notification(title="B", message="Msg 2", read=True)
        nl = NotificationList(
            notifications=[n1, n2],
            unread_count=1,
        )
        assert len(nl.notifications) == 2
        assert nl.unread_count == 1
