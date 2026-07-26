import { Link } from 'react-router';
import { Home } from 'lucide-react';

export function NotFoundPage() {
  return (
    <div className="flex min-h-[60vh] flex-col items-center justify-center gap-6 text-center">
      {/* 404 icon */}
      <div className="flex items-center gap-2">
        <span className="text-[120px] font-bold leading-none text-accent/20">404</span>
      </div>

      <div className="flex flex-col gap-2">
        <h1 className="text-2xl font-bold text-text-primary">Page not found</h1>
        <p className="max-w-sm text-sm text-text-secondary">
          The page you're looking for doesn't exist or has been moved.
        </p>
      </div>

      <Link
        to="/dashboard"
        className="inline-flex items-center gap-2 rounded-md bg-accent px-4 py-2 text-sm font-medium text-text-inverse transition-colors hover:bg-accent-hover"
      >
        <Home className="h-4 w-4" />
        Back to Dashboard
      </Link>
    </div>
  );
}
