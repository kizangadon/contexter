"""Notification domain models."""

from datetime import datetime, timezone
from typing import Optional
from uuid import UUID, uuid4

from pydantic import BaseModel, Field


class Notification(BaseModel):
    """A notification event for the user interface."""

    id: UUID = Field(default_factory=uuid4)
    title: str
    message: str
    type: str = "info"  # info, warning, error, success
    read: bool = False
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
    metadata: dict = Field(default_factory=dict)


class NotificationList(BaseModel):
    """A list of notifications with unread count."""

    notifications: list[Notification] = Field(default_factory=list)
    unread_count: int = 0
