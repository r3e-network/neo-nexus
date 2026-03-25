import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import Layout from './components/Layout';
import Dashboard from './pages/Dashboard';
import Nodes from './pages/Nodes';
import NodeDetail from './pages/NodeDetail';
import CreateNode from './pages/CreateNode';
import ImportNode from './pages/ImportNode';
import Plugins from './pages/Plugins';
import Settings from './pages/Settings';
import Login from './pages/Login';
import PublicDashboard from './pages/PublicDashboard';
import { WebSocketProvider } from './hooks/useWebSocket';
import { AuthProvider, useAuth } from './hooks/useAuth';

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

function AppRoutes() {
  return (
    <Routes>
      {/* Public Status Page - No login required */}
      <Route
        path="/status"
        element={
          <PublicRoute>
            <PublicDashboard />
          </PublicRoute>
        }
      />

      {/* Login Page */}
      <Route
        path="/login"
        element={
          <LoginRoute>
            <Login />
          </LoginRoute>
        }
      />

      {/* Protected Routes - Require authentication */}
      <Route
        path="/"
        element={
          <ProtectedRoute>
            <Layout>
              <Dashboard />
            </Layout>
          </ProtectedRoute>
        }
      />
      <Route
        path="/nodes"
        element={
          <ProtectedRoute>
            <Layout>
              <Nodes />
            </Layout>
          </ProtectedRoute>
        }
      />
      <Route
        path="/nodes/create"
        element={
          <ProtectedRoute>
            <Layout>
              <CreateNode />
            </Layout>
          </ProtectedRoute>
        }
      />
      <Route
        path="/nodes/import"
        element={
          <ProtectedRoute>
            <Layout>
              <ImportNode />
            </Layout>
          </ProtectedRoute>
        }
      />
      <Route
        path="/nodes/:id"
        element={
          <ProtectedRoute>
            <Layout>
              <NodeDetail />
            </Layout>
          </ProtectedRoute>
        }
      />
      <Route
        path="/plugins"
        element={
          <ProtectedRoute>
            <Layout>
              <Plugins />
            </Layout>
          </ProtectedRoute>
        }
      />
      <Route
        path="/settings"
        element={
          <ProtectedRoute>
            <Layout>
              <Settings />
            </Layout>
          </ProtectedRoute>
        }
      />
    </Routes>
  );
}

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <AuthProvider>
        <WebSocketProvider>
          <BrowserRouter>
            <AppRoutes />
          </BrowserRouter>
        </WebSocketProvider>
      </AuthProvider>
    </QueryClientProvider>
  );
}

export default App;
