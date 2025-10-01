import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { ReactQueryDevtools } from '@tanstack/react-query-devtools';
import { BrowserRouter, Route, Routes } from 'react-router-dom';

const queryClient = new QueryClient();

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <BrowserRouter>
        <div className="min-h-screen bg-gray-50">
          <header className="bg-honey-500 text-white p-4">
            <h1 className="text-2xl font-bold">🍯 HoneyLink™</h1>
            <p className="text-sm">Secure Device Session Platform</p>
          </header>
          <main className="container mx-auto p-4">
            <Routes>
              <Route path="/" element={<HomePage />} />
            </Routes>
          </main>
        </div>
      </BrowserRouter>
      <ReactQueryDevtools initialIsOpen={false} />
    </QueryClientProvider>
  );
}

function HomePage() {
  return (
    <div className="max-w-4xl mx-auto">
      <h2 className="text-3xl font-bold mb-4">Welcome to HoneyLink™</h2>
      <p className="text-gray-700 mb-4">
        The next-generation secure multi-device session platform, built with Rust + TypeScript.
      </p>
      <div className="bg-white rounded-soft shadow-md p-6">
        <h3 className="text-xl font-semibold mb-2">🚀 Getting Started</h3>
        <ul className="list-disc list-inside space-y-2 text-gray-600">
          <li>Pure Rust backend (no C/C++ dependencies)</li>
          <li>TypeScript + React frontend</li>
          <li>Zero Trust security architecture</li>
          <li>OpenTelemetry observability</li>
        </ul>
      </div>
    </div>
  );
}

export default App;
