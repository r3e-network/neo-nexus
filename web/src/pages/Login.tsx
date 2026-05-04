import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { useAuth } from '../hooks/useAuth';
import { Shield, Lock, User, Loader2, AlertTriangle } from 'lucide-react';
import { FeedbackBanner } from '../components/FeedbackBanner';
import { ApiRequestError } from '../utils/api';

export default function Login() {
  const { login } = useAuth();
  const navigate = useNavigate();
  const [username, setUsername] = useState('admin');
  const [password, setPassword] = useState('');
  const [error, setError] = useState('');
  const [suggestion, setSuggestion] = useState('');
  const [code, setCode] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [checkingSetup, setCheckingSetup] = useState(true);
  const [controlPlaneOffline, setControlPlaneOffline] = useState(false);

  useEffect(() => {
    const checkSetupStatus = async () => {
      try {
        const response = await fetch('/api/auth/setup-status');
        if (response.ok) {
          const data = (await response.json()) as { needsSetup: boolean };
          setControlPlaneOffline(false);
          if (data.needsSetup) {
            navigate('/setup', { replace: true });
            return;
          }
        } else if (response.status >= 500) {
          setControlPlaneOffline(true);
        }
      } catch {
        setControlPlaneOffline(true);
      } finally {
        setCheckingSetup(false);
      }
    };

    checkSetupStatus();
  }, [navigate]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');
    setSuggestion('');
    setCode('');
    setIsLoading(true);

    try {
      await login(username, password);
    } catch (err) {
      if (err instanceof ApiRequestError) {
        setError(err.message);
        setSuggestion(err.suggestion ?? '');
        setCode(err.code ?? '');
      } else {
        setError(err instanceof Error ? err.message : 'Authentication failed');
        setSuggestion('');
        setCode('');
      }
    } finally {
      setIsLoading(false);
    }
  };

  if (checkingSetup) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-slate-50">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500" />
      </div>
    );
  }

  return (
    <div className="flex min-h-screen items-center justify-center bg-slate-50 p-4">
      <div className="w-full max-w-[calc(100vw-2rem)] sm:max-w-lg animate-scale-in">
        <div className="mb-7 text-center">
          <div className="mx-auto mb-4 flex h-12 w-12 items-center justify-center rounded-lg border border-teal-200 bg-teal-50">
            <Shield className="h-6 w-6 text-teal-700" />
          </div>
          <h1 className="text-2xl font-semibold text-slate-950">NeoNexus</h1>
          <p className="mt-2 text-sm font-medium uppercase text-slate-500">Node operations console</p>
        </div>

        {/* Control plane offline */}
        {controlPlaneOffline && (
          <div className="mb-6 rounded-lg border border-red-200 bg-red-50 p-4 text-sm text-red-800">
            <div className="flex items-start gap-3">
              <AlertTriangle className="mt-0.5 h-5 w-5 shrink-0 text-red-700" />
              <div>
                <p className="font-semibold text-red-950">Control plane API is offline</p>
                <p className="mt-1 leading-6">
                  The web console is loaded, but NeoNexus server APIs are not reachable. Start the full stack with <code className="rounded bg-red-100 px-1.5 py-0.5">npm run dev</code> or run the API server on port 8080 before signing in.
                </p>
              </div>
            </div>
          </div>
        )}

        <div className="mb-5 rounded-lg border border-blue-200 bg-blue-50 p-4">
          <div className="flex items-start gap-3">
            <Shield className="mt-0.5 h-5 w-5 shrink-0 text-blue-700" />
            <div className="min-w-0 text-sm text-blue-900">
              <p className="font-semibold text-blue-950">Secure sign-in</p>
              <p className="mt-1 leading-5 text-blue-800">
                Use the admin account created during first-time setup. New installs are redirected to setup before sign-in.
              </p>
            </div>
          </div>
        </div>

        <div className="card">
          <FeedbackBanner error={error} suggestion={suggestion} code={code} />

          <form onSubmit={handleSubmit} className="space-y-4">
            <div>
              <label htmlFor="login-username" className="mb-2 block text-sm font-medium text-slate-700">
                Username
              </label>
              <div className="relative">
                <User className="pointer-events-none absolute left-3.5 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-500" />
                <input
                  id="login-username"
                  type="text"
                  value={username}
                  onChange={(e) => setUsername(e.target.value)}
                  className="input h-11 pl-12"
                  placeholder="Enter username"
                  autoComplete="username"
                  required
                />
              </div>
            </div>

            <div>
              <label htmlFor="login-password" className="mb-2 block text-sm font-medium text-slate-700">
                Password
              </label>
              <div className="relative">
                <Lock className="pointer-events-none absolute left-3.5 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-500" />
                <input
                  id="login-password"
                  type="password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  className="input h-11 pl-12"
                  placeholder="Enter password"
                  autoComplete="current-password"
                  required
                />
              </div>
            </div>

            <button
              type="submit"
              disabled={isLoading}
              className="btn btn-primary h-11 w-full justify-center"
            >
              {isLoading ? (
                <>
                  <Loader2 className="w-4 h-4 animate-spin" />
                  Signing in...
                </>
              ) : (
                'Sign In'
              )}
            </button>
          </form>
        </div>

        <div className="mt-6 text-center text-sm text-slate-400">
          <p>NeoNexus Node Manager v{__APP_VERSION__}</p>
          <p className="mt-1 text-slate-500">Secure your node with strong credentials</p>
        </div>
      </div>
    </div>
  );
}
