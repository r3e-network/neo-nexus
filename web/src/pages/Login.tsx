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
      <div className="min-h-screen flex items-center justify-center bg-slate-950">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500" />
      </div>
    );
  }

  return (
    <div className="flex min-h-screen items-center justify-center bg-[radial-gradient(circle_at_50%_-10%,rgba(113,112,255,0.16),transparent_32rem)] p-4">
      <div className="w-full max-w-lg animate-scale-in">
        <div className="mb-7 text-center">
          <div className="mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-[1.25rem] border border-indigo-200/15 bg-indigo-300/10 shadow-[0_18px_48px_-28px_rgba(113,112,255,0.9)]">
            <Shield className="h-8 w-8 text-indigo-200" />
          </div>
          <h1 className="text-3xl font-semibold tracking-[-0.05em] text-white"><span className="bg-gradient-to-r from-indigo-200 via-white to-cyan-200 bg-clip-text text-transparent">NeoNexus</span></h1>
          <p className="mt-2 text-sm font-medium uppercase tracking-[0.22em] text-slate-500">Node operations console</p>
        </div>

        {/* Control plane offline */}
        {controlPlaneOffline && (
          <div className="mb-6 rounded-2xl border border-red-400/20 bg-red-500/10 p-4 text-sm text-red-100 shadow-lg">
            <div className="flex items-start gap-3">
              <AlertTriangle className="mt-0.5 h-5 w-5 shrink-0 text-red-300" />
              <div>
                <p className="font-semibold text-white">Control plane API is offline</p>
                <p className="mt-1 leading-6 text-red-100/85">
                  The web console is loaded, but NeoNexus server APIs are not reachable. Start the full stack with <code className="rounded bg-slate-950/60 px-1.5 py-0.5">npm run dev</code> or run the API server on port 8080 before signing in.
                </p>
              </div>
            </div>
          </div>
        )}

        {/* Default Credentials Warning */}
        <div className="mb-5 rounded-2xl border border-amber-300/18 bg-[linear-gradient(135deg,rgba(245,158,11,0.10),rgba(255,255,255,0.025))] p-4 shadow-[0_18px_48px_-36px_rgba(245,158,11,0.65)]">
          <div className="flex items-start gap-3">
            <AlertTriangle className="mt-0.5 h-5 w-5 shrink-0 text-amber-300" />
            <div className="min-w-0 text-sm text-amber-100/90">
              <div className="mb-2 flex flex-wrap items-center gap-2">
                <p className="font-semibold text-amber-100">Initial access hint</p>
                <span className="rounded-full border border-amber-200/20 bg-amber-300/10 px-2 py-0.5 text-[11px] uppercase tracking-[0.14em] text-amber-200/80">local setup</span>
              </div>
              <div className="grid gap-2 sm:grid-cols-2">
                <p className="rounded-xl border border-white/[0.06] bg-slate-950/35 px-3 py-2 text-slate-300">Username <strong className="float-right text-white">admin</strong></p>
                <p className="rounded-xl border border-white/[0.06] bg-slate-950/35 px-3 py-2 text-slate-300">Password <strong className="float-right text-white">admin</strong></p>
              </div>
              <p className="mt-3 leading-5 text-amber-100/72">
                Change the default password immediately after first login in Settings → Change Password.
              </p>
            </div>
          </div>
        </div>

        <div className="card">
          <FeedbackBanner error={error} suggestion={suggestion} code={code} />

          <form onSubmit={handleSubmit} className="space-y-4">
            <div>
              <label className="mb-2 block text-sm font-medium text-slate-300">
                Username
              </label>
              <div className="relative">
                <User className="pointer-events-none absolute left-3.5 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-500" />
                <input
                  type="text"
                  value={username}
                  onChange={(e) => setUsername(e.target.value)}
                  className="input h-11 pl-12"
                  placeholder="Enter username"
                  required
                />
              </div>
            </div>

            <div>
              <label className="mb-2 block text-sm font-medium text-slate-300">
                Password
              </label>
              <div className="relative">
                <Lock className="pointer-events-none absolute left-3.5 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-500" />
                <input
                  type="password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  className="input h-11 pl-12"
                  placeholder="Enter password"
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
