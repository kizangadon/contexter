"""FastAPI router for Agent CRUD operations."""

from fastapi import APIRouter, Depends, HTTPException, status

from contexter_server.models.agent import Agent, AgentCreate, AgentPatch
from contexter_server.services.agent_service import AgentService
from .deps import _validate_id_length, get_agent_service


router = APIRouter(prefix="/api/v1/agents", tags=["agents"])


@router.get("", response_model=list[Agent])
async def list_agents(
    service: AgentService = Depends(get_agent_service),
) -> list[Agent]:
    """List all agents."""
    return await service.list()


@router.post("", response_model=Agent, status_code=status.HTTP_201_CREATED)
async def create_agent(
    data: AgentCreate,
    service: AgentService = Depends(get_agent_service),
) -> Agent:
    """Create a new agent configuration."""
    return await service.create(data)


@router.get("/{id}", response_model=Agent)
async def get_agent(
    id: str,
    service: AgentService = Depends(get_agent_service),
) -> Agent:
    """Get an agent by ID."""
    _validate_id_length(id)
    agent = await service.get(id)
    if not agent:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Agent not found",
        )
    return agent


@router.put("/{id}", response_model=Agent)
async def update_agent(
    patch: AgentPatch,
    id: str,
    service: AgentService = Depends(get_agent_service),
) -> Agent:
    """Update an agent by ID (partial update)."""
    _validate_id_length(id)
    agent = await service.update(id, patch)
    if not agent:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Agent not found",
        )
    return agent


@router.delete("/{id}", status_code=status.HTTP_204_NO_CONTENT)
async def delete_agent(
    id: str,
    service: AgentService = Depends(get_agent_service),
) -> None:
    """Delete an agent by ID (idempotent)."""
    _validate_id_length(id)
    await service.delete(id)
