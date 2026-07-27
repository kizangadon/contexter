# AC-001: Shadow rgba has no spaces
Given tokens.css, When checking shadow rgba values, Then they match `rgba(0,0,0,0.3)` format without spaces.

# AC-002: Build passes
Given the formatting changes, When npm run build runs, Then it succeeds.
