# Bug 9 Design Preview — Module Structure

## Changes
1. `git mv src/error.rs src/error/mod.rs` — directory-based module
2. `git mv src/cli.rs src/cli/mod.rs` — directory-based module
3. `src/lib.rs`: Replace `pub use models::*;` with explicit re-exports
