import { createBrowserRouter, RouterProvider, Navigate } from 'react-router';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { RootLayout } from './components/layout/RootLayout';
import { ToastProvider } from './components/ui/ToastProvider';
import { routes } from './routes';

/* ─── Query client ────────────────────────────────────────────── */

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 30_000,
      retry: 1,
      refetchOnWindowFocus: false,
    },
  },
});

/* ─── Router ──────────────────────────────────────────────────── */

const router = createBrowserRouter([
  {
    element: <RootLayout />,
    errorElement: <Navigate to="/" replace />,
    children: [
      { index: true, element: <Navigate to="/dashboard" replace /> },
      ...routes,
    ],
  },
]);

/* ─── App component ───────────────────────────────────────────── */

export function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <ToastProvider>
        <RouterProvider router={router} />
      </ToastProvider>
    </QueryClientProvider>
  );
}
