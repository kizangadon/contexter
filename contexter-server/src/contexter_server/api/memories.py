"""FastAPI router for Memory CRUD, search, and versioning."""

from fastapi import APIRouter, Depends, HTTPException, Query, status

from contexter_server.models.memory import Memory, MemoryCreate, MemoryPatch
from contexter_server.models.search import SearchQuery, SearchResponse
from contexter_server.services.memory_service import MemoryService
from .deps import _validate_id_length, get_memory_service


router = APIRouter(prefix="/api/v1/memories", tags=["memories"])


@router.get("", response_model=list[Memory])
async def list_memories(
    service: MemoryService = Depends(get_memory_service),
) -> list[Memory]:
    """List all memories."""
    return await service.list()


@router.post("", response_model=Memory, status_code=status.HTTP_201_CREATED)
async def create_memory(
    data: MemoryCreate,
    service: MemoryService = Depends(get_memory_service),
) -> Memory:
    """Create a new memory entry."""
    return await service.create(data)


@router.get("/search", response_model=SearchResponse)
async def search_memories(
    q: str = Query(..., description="Search query string"),
    session_id: str | None = Query(None, description="Filter by session ID"),
    type_filter: str | None = Query(None, alias="type", description="Filter by type"),
    limit: int = Query(20, ge=1, le=100, description="Max results"),
    page: int = Query(1, ge=1, description="Page number"),
    service: MemoryService = Depends(get_memory_service),
) -> SearchResponse:
    """Search memories by query string."""
    query = SearchQuery(
        query=q,
        session_id=session_id,
        type=type_filter,
        limit=limit,
        page=page,
    )
    return await service.search(query)


@router.get("/{id}", response_model=Memory)
async def get_memory(
    id: str,
    service: MemoryService = Depends(get_memory_service),
) -> Memory:
    """Get a memory by ID."""
    _validate_id_length(id)
    memory = await service.get(id)
    if not memory:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Memory not found",
        )
    return memory


@router.put("/{id}", response_model=Memory)
async def update_memory(
    patch: MemoryPatch,
    id: str,
    service: MemoryService = Depends(get_memory_service),
) -> Memory:
    """Update a memory by ID (partial update)."""
    _validate_id_length(id)
    memory = await service.update(id, patch)
    if not memory:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Memory not found",
        )
    return memory


@router.delete("/{id}", status_code=status.HTTP_204_NO_CONTENT)
async def delete_memory(
    id: str,
    service: MemoryService = Depends(get_memory_service),
) -> None:
    """Delete a memory by ID (idempotent)."""
    _validate_id_length(id)
    await service.delete(id)


@router.post("/{id}/versions", status_code=status.HTTP_201_CREATED)
async def create_memory_version(
    id: str,
    service: MemoryService = Depends(get_memory_service),
) -> dict:
    """Create a new version of a memory (placeholder)."""
    _validate_id_length(id)
    memory = await service.get(id)
    if not memory:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Memory not found",
        )
    # TODO: implement version creation when bridge supports it
    return {"status": "version_created", "memory_id": id}
