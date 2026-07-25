"""Pydantic models for user feedback: bug reports and feature suggestions."""

from typing import Optional

from pydantic import BaseModel, Field


class BugReport(BaseModel):
    """A validated bug report submitted by a user."""

    title: str = Field(..., min_length=1, description="Short summary of the bug")
    description: str = Field(..., min_length=1, description="Detailed description of the bug")
    email: Optional[str] = Field(None, description="Contact email of the reporter")
    severity: str = Field("medium", description="Severity level: low, medium, high, critical")
    category: str = Field("general", description="Functional category of the bug")


class FeatureSuggestion(BaseModel):
    """A validated feature suggestion submitted by a user."""

    title: str = Field(..., min_length=1, description="Short summary of the suggestion")
    description: str = Field(..., min_length=1, description="Detailed description of the suggestion")
    email: Optional[str] = Field(None, description="Contact email of the suggester")
    severity: str = Field("medium", description="Perceived importance: low, medium, high")
    category: str = Field("general", description="Functional category of the suggestion")
