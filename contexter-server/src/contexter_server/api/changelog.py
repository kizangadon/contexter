"""FastAPI router for changelog listing."""

from fastapi import APIRouter

router = APIRouter(prefix="/api/v1/changelog", tags=["changelog"])


@router.get("")
async def list_changelog() -> list[dict]:
    """List changelog entries."""
    # TODO: implement changelog from git history or config
    return []
