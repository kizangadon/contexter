# Bug: UI Primitives Missing Variants and Components

## Issues
1. **Button** only has `primary/secondary/ghost` variants but design specifies `danger` variant + `sm`/`lg` sizes
2. **SearchInput** is listed as a shared UI primitive but doesn't exist as a separate component (only `Input` exists)
3. **SidebarNav** has `NavItem.section` field that is never rendered — NavSection labels for grouped items are missing from the wireframe

## Fix
1. **Button.tsx**: Add `danger` variant (red/destructive styling), add `size` prop with `sm`/`md`/`lg` values, update type
2. **SearchInput.tsx**: Create new component wrapping Input with search icon, clear button, keyboard shortcut hint
3. **SidebarNav.tsx**: Render section group labels when items have a `section` field, showing dividers between sections
