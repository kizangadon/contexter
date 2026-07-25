# Acceptance Criteria

### AC-01: Doc comment accurate
GIVEN the DuckDbEngine struct doc comment  
WHEN read  
THEN it MUST NOT mention "two separate connections" and MUST accurately describe the single Mutex<Connection> pattern

### AC-02: Limitation documented
GIVEN the DuckDbEngine struct  
WHEN the doc comment is read  
THEN it MUST document that the single connection is a known limitation and incremental sync mitigates write duration
