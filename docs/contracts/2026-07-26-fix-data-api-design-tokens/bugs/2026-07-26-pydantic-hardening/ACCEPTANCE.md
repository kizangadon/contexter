# AC-001: null sessionId accepted
Given Memory.model_validate receives `{"sessionId": null}`, When validating, Then no ValidationError is raised and session_id is None.

# AC-002: ConfigDict comment present
Given memory.py and session.py, When inspecting, Then each has a comment explaining populate_by_name=True.
