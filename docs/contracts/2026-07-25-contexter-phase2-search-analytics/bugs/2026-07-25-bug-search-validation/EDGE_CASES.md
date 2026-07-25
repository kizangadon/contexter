# Edge Cases — Bug-Search-Validation

- EC-01: `vector_weight = -0.0` (negative zero) — handled by clamp
- EC-02: `vector_weight = NaN` — not explicitly checked as it's f32; clamp to [0,1] is NaN-transparent
- EC-03: `limit = u32::MAX` — capped to 1000
- EC-04: `sort_field = "  "` (whitespace) — treat as empty
