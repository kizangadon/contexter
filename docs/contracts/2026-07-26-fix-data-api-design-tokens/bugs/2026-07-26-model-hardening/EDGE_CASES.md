# Edge Cases

- embedding: model_serializer must not break model_validate from dict/JSON
- embedding: must not affect model_dump(exclude=...) or model_dump(include=...) behavior
- datetime: timezone-aware datetimes must pass through unchanged
- datetime: non-datetime values (strings) must still work with Pydantic's built-in coercion
- status: "done" → "completed" normalization is one-way; no reverse mapping
