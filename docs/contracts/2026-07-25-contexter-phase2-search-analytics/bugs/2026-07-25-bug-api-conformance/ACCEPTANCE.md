# Acceptance Criteria

### AC-01: Field names match design
GIVEN the `HybridSearchQuery` struct  
WHEN inspected  
THEN its field names MUST match the design preview (`query_text`, `query_vector`, `top_k`, `text_weight`, etc.)

### AC-02: FTS has entity schemas
GIVEN the FTS index  
WHEN queried for entity-specific schemas  
THEN schemas for session, agent, and skill exist with correct fields and boosts

### AC-03: Cache policy matches design
GIVEN `create_memory` is called  
WHEN the L1 cache is checked  
THEN the old entry MUST be invalidated (not write-through updated)

### AC-04: Field boosts match design
GIVEN the FTS memory schema  
WHEN field boosts are inspected  
THEN they match the design preview values
