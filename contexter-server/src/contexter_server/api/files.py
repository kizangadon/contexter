"""FastAPI router for file listing, diffing, and watching."""

import os
from pathlib import Path

from fastapi import APIRouter, HTTPException, Query, status
from pydantic import BaseModel, Field


class WatchFilesRequest(BaseModel):
    """Validated request to start watching a file or directory."""

    path: str = Field(..., min_length=1, description="File or directory path to watch")
    recursive: bool = Field(False, description="Watch subdirectories recursively")
    events: list[str] = Field(
        default_factory=lambda: ["create", "modify"],
        min_length=1,
        description="List of event types to watch (create, modify, delete)",
    )


# ---------------------------------------------------------------------------
# Path traversal protection
# ---------------------------------------------------------------------------


def validate_safe_path(path: str, base_dir: str | None = None) -> Path:
    """Validate and resolve a file system path, rejecting traversal attacks.

    Resolves the path to an absolute form and rejects any path that
    contains ``..`` components.  When *base_dir* is provided, also
    verifies that the resolved path falls inside *base_dir*.

    Parameters
    ----------
    path:
        User-supplied path to validate.
    base_dir:
        Optional base directory to confine access to.  Paths that
        resolve outside *base_dir* are rejected with ``403``.

    Returns
    -------
    Path
        Resolved absolute :class:`Path` that is safe to use.

    Raises
    ------
    HTTPException(400)
        If the path is invalid or attempts traversal.
    HTTPException(403)
        If the resolved path is outside *base_dir*.
    """
    # Check for raw ``..`` in path components
    sep = os.sep
    if ".." in path.split(sep):
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="Path must not contain '..' components",
        )
    # Reject URL-encoded ``..`` (%2e%2e, %2E%2E)
    normalized = path.replace("\\", "/")
    if "%2e" in normalized.lower():
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="Path must not contain '..' components",
        )

    abs_path = Path(os.path.abspath(path))

    # Base-directory confinement
    if base_dir is not None:
        resolved_base = os.path.abspath(base_dir)
        if os.path.commonpath([str(abs_path), resolved_base]) != resolved_base:
            raise HTTPException(
                status_code=status.HTTP_403_FORBIDDEN,
                detail="Path outside allowed directory",
            )

    return abs_path


# ---------------------------------------------------------------------------
# Router
# ---------------------------------------------------------------------------

router = APIRouter(prefix="/api/v1/files", tags=["files"])


@router.get("")
async def list_files(
    path: str = Query(".", description="Directory path to list"),
) -> dict:
    """List files in the given path."""
    validate_safe_path(path, base_dir=os.getcwd())
    # TODO: implement file listing when bridge supports it
    return {"path": path, "files": [], "total": 0}


@router.get("/{hash}/diff")
async def file_diff(hash: str, base: str = Query(...), compare: str = Query(...)) -> dict:
    """Compute a diff between two file versions by content hash."""
    # TODO: implement file diff when bridge supports it
    # TODO: validate base/compare with validate_safe_path()
    return {
        "hash": hash,
        "base": base,
        "compare": compare,
        "changes": [],
        "summary": {"added": 0, "removed": 0, "modified": 0},
    }


@router.post("/watch")
async def watch_files(body: WatchFilesRequest) -> dict:
    """Start watching a file or directory for changes."""
    # TODO: implement file watching when bridge supports it
    return {"status": "watching", "path": body.path, "watcher_id": ""}
