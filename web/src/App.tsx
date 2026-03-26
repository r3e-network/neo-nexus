import { Component, type ReactNode } from 'react';
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import Layout from './components/Layout';
import Dashboard from './pages/Dashboard';
import Nodes from './pages/Nodes';
import NodeDetail from './pages/NodeDetail';
import CreateNode from './pages/CreateNode';
import ImportNode from './pages/ImportNode';
import Servers from './pages/Servers';
import Plugins from './pages/Plugins';
import Settings from './pages/Settings';
import Login from './pages/Login';
import PublicDashboard from './pages/PublicDashboard';
import { WebSocketProvider } from './hooks/useWebSocket';
import { AuthProvider, useAuth } from './hooks/useAuth';
import { NotificationsProvider } from './hooks/useNotifications';

class ErrorBoundary extends Component<
  { children: ReactNode },
  { hasError: boolean; error: Error | null }
> {
  constructor(props: { children: ReactNode }) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error) {
    return { hasError: true, error };
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="min-h-screen flex items-center justify-center bg-slate-950 p-8">
          <div className="max-w-md text-center">
            <h1 className="text-2xl font-bold text-white mb-4">Something went wrong</h1>
            <p className="text-slate-400 mb-6">{this.state.error?.message || 'An unexpected error occurred.'}</p>
            <button
              onClick={() => {
                this.setState({ hasError: false, error: null });
                window.location.href = '/';
              }}
              className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-500 transition-colors"
            >
              Return to Dashboard
            </button>
          </div>
        </div>
      );
    }
    return this.props.children;
  }
}

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      refetchInterval: 5000,
      retry: 2,
    },
  },
});

// Protected route wrapper - requires authentication
function ProtectedRoute({ children }: { children: React.ReactNode }) {
  const { user, isLoading } = useAuth();

  if (isLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-slate-950">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500" />
      </div>
    );
  }

  if (!user) {
    return <Navigate to="/login" replace />;
  }

  return <>{children}</>;
}

// Public route wrapper - accessible without authentication
function PublicRoute({ children }: { children: React.ReactNode }) {
  return <>{children}</>;
}

// Login route - redirects to dashboard if already logged in
function LoginRoute({ children }: { children: React.ReactNode }) {
  const { user, isLoading } = useAuth();

  if (isLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-slate-950">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500" />
      </div>
    );
  }

  if (user) {
    return <Navigate to="/" replace />;
  }

  return <>{children}</>;
}

function ProtectedPage({ children }: { children: React.ReactNode }) {
  return (
    <ProtectedRoute>
      <Layout>{children}</Layout>
    </ProtectedRoute>
  );
}

function AppRoutes() {
  return (
    <Routes>
      <Route path="/status" element={<PublicRoute><PublicDashboard /></PublicRoute>} />
      <Route path="/login" element={<LoginRoute><Login /></LoginRoute>} />

      <Route path="/" element={<ProtectedPage><Dashboard /></ProtectedPage>} />
      <Route path="/nodes" element={<ProtectedPage><Nodes /></ProtectedPage>} />
      <Route path="/nodes/create" element={<ProtectedPage><CreateNode /></ProtectedPage>} />
      <Route path="/nodes/import" element={<ProtectedPage><ImportNode /></ProtectedPage>} />
      <Route path="/nodes/:id" element={<ProtectedPage><NodeDetail /></ProtectedPage>} />
      <Route path="/servers" element={<ProtectedPage><Servers /></ProtectedPage>} />
      <Route path="/plugins" element={<ProtectedPage><Plugins /></ProtectedPage>} />
      <Route path="/settings" element={<ProtectedPage><Settings /></ProtectedPage>} />
    </Routes>
  );
}

function App() {
  return (
    <ErrorBoundary>
      <QueryClientProvider client={queryClient}>
        <AuthProvider>
          <WebSocketProvider>
            <NotificationsProvider>
              <BrowserRouter>
                <AppRoutes />
              </BrowserRouter>
            </NotificationsProvider>
          </WebSocketProvider>
        </AuthProvider>
      </QueryClientProvider>
    </ErrorBoundary>
  );
}

export default App;
