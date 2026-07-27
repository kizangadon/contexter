# Acceptance Criteria — Fix Data API + Align Design Tokens

## Happy Path

### AC-001 — Memory list returns data
- **Given** the Rust engine has 194 stored memories
- **When** `GET /api/v1/memories` is called
- **Then** the response SHALL be a non-empty JSON array (status 200)

### AC-002 — Memory response fields correct
- **Given** a memory object is returned by the API
- **When** inspecting its fields
- **Then** it SHALL contain: `id`, `session_id`, `agent_id`, `memory_type`, `content`, `tags`, `version`, `created_at`, `updated_at`
- **And** optional fields `embedding`, `tokens`, `tokenizer`, `model`, `metadata`, `role` SHALL be present with `null`/default values when Rust does not emit them

### AC-003 — Session list returns data
- **Given** the Rust engine has 1 stored session
- **When** `GET /api/v1/sessions` is called
- **Then** the response SHALL be a non-empty JSON array (status 200)

### AC-004 — Session response fields correct
- **Given** a session object is returned by the API
- **When** inspecting its fields
- **Then** it SHALL contain: `id`, `agent_id`, `project`, `status`, `turn_count`, `duration_ms`, `started_at`, `last_active`
- **And** optional field `efficiency_score` SHALL be present
- **And** `metadata` SHALL be a JSON object

### AC-005 — Token colors match V2-DEEP spec
- **Given** the approved V2-DEEP-design-system.md
- **When** checking each color token in `tokens.css`
- **Then** all hex values, alpha values, and gradient strings SHALL match exactly

### AC-006 — All token groups present
- **Given** the V2-DEEP-design-system.md lists: shadows, gradients, chart colors, motion, layout, type scale, semantic backgrounds, surface cards
- **When** inspecting `tokens.css`
- **Then** every group SHALL have at least one token defined

## Error/Edge Cases

### AC-007 — Memory list handles empty engine
- **Given** the Rust engine has zero memories
- **When** `GET /api/v1/memories` is called
- **Then** the response SHALL be an empty array `[]` (not `null`, not an error)

### AC-008 — Session list handles empty engine
- **Given** the Rust engine has zero sessions
- **When** `GET /api/v1/sessions` is called
- **Then** the response SHALL be an empty array `[]`

### AC-009 — Unknown Rust fields silently ignored
- **Given** the Rust engine adds a new field not in the Pydantic model
- **When** `Memory.model_validate(raw)` is called
- **Then** the extra field SHALL be silently ignored (not raise ValidationError)

### AC-010 — Token values use correct CSS syntax
- **Given** tokens.css defines custom properties
- **When** validated by the browser/CSS parser
- **Then** no invalid property values exist (e.g. malformed gradients, invalid color strings)
