"""FastAPI router for user feedback submission."""

from fastapi import APIRouter

from contexter_server.models.feedback import BugReport, FeatureSuggestion

router = APIRouter(prefix="/api/v1/feedback", tags=["feedback"])


@router.post("/bug", status_code=201)
async def report_bug(body: BugReport) -> dict:
    """Submit a bug report."""
    # TODO: implement bug report persistence
    return {"status": "received", "type": "bug", "message": "Bug report submitted"}


@router.post("/suggest", status_code=201)
async def suggest_feature(body: FeatureSuggestion) -> dict:
    """Submit a feature suggestion."""
    # TODO: implement suggestion persistence
    return {"status": "received", "type": "suggestion", "message": "Suggestion submitted"}
