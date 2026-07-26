# Bug: App.tsx is a placeholder — app fails to bootstrap

## Issue
`App.tsx` renders `<div><p>Contexter</p></div>` instead of the full SPA shell. The component architecture from the design preview shows:

```
App (QueryClientProvider + RouterProvider) → AppShell (Sidebar + TopBar + Outlet) → Pages
```

But:
- No `QueryClientProvider` wraps the app → all `@tanstack/react-query` hooks will throw `No QueryClient set`
- No `RouterProvider` connects the routes → React Router v7 routes are defined but never rendered
- `AppShell` component exists but is never used → no sidebar, no top bar, no layout
- Routes exist in `routes.tsx` but are not connected to any router

## Fix
1. Rewrite `App.tsx` to:
   - Create a `QueryClient` from `@tanstack/react-query`
   - Wrap everything in `<QueryClientProvider>`
   - Create a `createBrowserRouter` from the routes in `routes.tsx`
   - Wrap the routes in an `AppShell` layout route with nav items
   - Pass the router to `<RouterProvider>`
2. Define nav items matching the sidebar sections from the design preview
3. Add the 12 missing sub-routes for efficiency/*, analytics/*, settings/* sub-pages

## Missing sub-routes to add
- `/efficiency/memory`, `/efficiency/sessions`, `/efficiency/agents`, `/efficiency/skills`, `/efficiency/tokens`, `/efficiency/correlation`
- `/analytics/health`, `/analytics/performance`, `/analytics/resources`, `/analytics/costs`, `/analytics/costs/models/:id`, `/analytics/services`
- 8 settings sub-routes: `/settings/general`, `/settings/storage`, `/settings/mcp`, `/settings/llm`, `/settings/notifications`, `/settings/agents-skills`, `/settings/analytics`, `/settings/data-management`

For missing pages, create simple placeholder components or redirect to the parent route.
