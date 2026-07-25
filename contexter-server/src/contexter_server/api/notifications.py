"""FastAPI router for notification management."""

from fastapi import APIRouter, Depends, HTTPException, status

from contexter_server.models.notifications import Notification, NotificationList
from contexter_server.services.notification_service import NotificationService

from .deps import get_notification_service

router = APIRouter(prefix="/api/v1/notifications", tags=["notifications"])


@router.get("", response_model=NotificationList)
async def list_notifications(
    service: NotificationService = Depends(get_notification_service),
) -> NotificationList:
    """List recent notifications with unread count."""
    return await service.list()


@router.put("/{id}/read", response_model=Notification)
async def mark_read(
    id: str,
    service: NotificationService = Depends(get_notification_service),
) -> Notification:
    """Mark a single notification as read."""
    notif = await service.mark_read(id)
    if notif is None:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Notification not found",
        )
    return notif


@router.post("/read-all", response_model=NotificationList)
async def mark_all_read(
    service: NotificationService = Depends(get_notification_service),
) -> NotificationList:
    """Mark all notifications as read."""
    return await service.mark_all_read()
