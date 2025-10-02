import { Home } from 'lucide-react';
import { Link } from 'react-router-dom';

/**
 * 404 Not Found page
 */
export const NotFoundPage = () => {
  return (
    <div className="flex min-h-[60vh] flex-col items-center justify-center space-y-6 text-center">
      <div className="text-6xl font-bold text-amber-500">404</div>
      <h1 className="text-3xl font-bold text-gray-900 dark:text-white">
        Page Not Found
      </h1>
      <p className="text-gray-600 dark:text-gray-400">
        The page you're looking for doesn't exist.
      </p>
      <Link
        to="/"
        className="flex items-center gap-2 rounded-lg bg-amber-500 px-6 py-3 font-medium text-white hover:bg-amber-600"
      >
        <Home className="h-5 w-5" />
        Back to Home
      </Link>
    </div>
  );
};
