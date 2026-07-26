# Acceptance Criteria

- AC-001: App.tsx wraps children with QueryClientProvider
- AC-002: App.tsx creates a createBrowserRouter from routes
- AC-003: App.tsx renders RouterProvider with the created router
- AC-004: Routes are nested under AppShell as a layout route with <Outlet />
- AC-005: AppShell receives correct navItems with all configured nav items and sections
- AC-006: All 27 existing routes still work after refactor
- AC-007: 12 missing sub-routes are added with at minimum placeholder/redirect pages
- AC-008: tsc --noEmit passes without errors
