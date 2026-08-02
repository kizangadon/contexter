# EDGE_CASES — Pydantic Alias Annotated
- EC-PAA-001: new-style payload (no legacy alias) parses identically.
- EC-PAA-002: both alias and canonical key present → canonical wins (unchanged pydantic semantics).
- EC-PAA-003: None/absent values remain Optional (no new validation errors).
