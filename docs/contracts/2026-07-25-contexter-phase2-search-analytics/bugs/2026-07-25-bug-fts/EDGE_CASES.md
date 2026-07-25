# Edge Cases — Bug-FTS

- EC-01: Empty title or tags — index with empty string for those fields
- EC-02: Very long tags string — Tantivy handles via its text field limits
- EC-03: Path doesn't exist when creating TantivyIndex — Tantivy creates it (existing behavior)
- EC-04: Alias names must be non-empty strings
- EC-05: `switch_index` to a non-existent alias — return error
