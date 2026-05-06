import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { Activity, AlertTriangle, CheckCircle2, KeyRound, Loader2, Lock, Server, ShieldCheck, User } from 'lucide-react';
import { api, ApiRequestError } from '../utils/api';
import { FeedbackBanner } from '../components/FeedbackBanner';
import { useAuth } from '../hooks/useAuth';
import { PasswordStrengthMeter } from '../components/PasswordStrengthMeter';

interface SetupResponse {
  token: string;
  user: { id: string; username: string; role: "admin" | "viewer" };
}

export default function Setup() {
  const navigate = useNavigate();
  const { setSession } = useAuth();
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
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
          if (!data.needsSetup) {
            navigate('/login', { replace: true });
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

    void checkSetupStatus();
  }, [navigate]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');
    setSuggestion('');
    setCode('');

    const normalizedUsername = username.trim();

    if (!normalizedUsername || !password || !confirmPassword) {
      setError('All fields are required');
      return;
    }

    if (normalizedUsername.length < 3) {
      setError('Username must be at least 3 characters');
      return;
    }

    if (password.length < 8) {
      setError('Password must be at least 8 characters');
      return;
    }

    if (password !== confirmPassword) {
      setError('Passwords do not match');
      return;
    }

    setIsLoading(true);

    try {
      const data = await api.post<SetupResponse>('/api/auth/setup', { username: normalizedUsername, password });
      setSession(data.token, data.user);
      navigate('/', { replace: true });
    } catch (err) {
      if (err instanceof ApiRequestError) {
        setError(err.message);
        setSuggestion(err.suggestion ?? '');
        setCode(err.code ?? '');
      } else {
        setError(err instanceof Error ? err.message : 'Setup failed');
        setSuggestion('');
        setCode('');
      }
    } finally {
      setIsLoading(false);
    }
  };

  if (checkingSetup) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-slate-50">
        <div className="flex items-center gap-3 rounded-lg border border-slate-200 bg-white px-4 py-3 text-sm text-slate-600 shadow-sm">
          <Loader2 className="h-4 w-4 animate-spin text-teal-700" />
          Checking setup state...
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-slate-50 p-4">
      <div className="mx-auto flex min-h-[calc(100vh-2rem)] w-full max-w-5xl flex-col justify-center gap-6 py-8 lg:grid lg:grid-cols-[minmax(0,0.9fr)_minmax(420px,1fr)] lg:items-center">
        <section className="rounded-lg border border-slate-200 bg-white p-6 shadow-sm">
          <div className="flex items-center gap-3">
            <div className="flex h-12 w-12 items-center justify-center rounded-lg border border-teal-200 bg-teal-50">
              <Activity className="h-6 w-6 text-teal-700" />
            </div>
            <div>
              <h1 className="text-2xl font-semibold text-slate-950">NeoNexus</h1>
              <p className="text-sm font-medium uppercase text-slate-500">First-run setup</p>
            </div>
          </div>

          <div className="mt-6 space-y-4 text-sm leading-6 text-slate-600">
            <p>
              Create the first administrator before managing nodes, plugins, private networks, fast sync data, and signer policy.
            </p>
            <div className="grid gap-3">
              <div className="flex items-start gap-3 rounded-lg border border-slate-200 bg-slate-50 p-3">
                <ShieldCheck className="mt-0.5 h-5 w-5 shrink-0 text-teal-700" />
                <div>
                  <p className="font-semibold text-slate-950">Local control plane</p>
                  <p>Keep this console bound to trusted hosts and use a strong admin password before exposing it beyond localhost.</p>
                </div>
              </div>
              <div className="flex items-start gap-3 rounded-lg border border-slate-200 bg-slate-50 p-3">
                <KeyRound className="mt-0.5 h-5 w-5 shrink-0 text-blue-700" />
                <div>
                  <p className="font-semibold text-slate-950">Admin-only bootstrap</p>
                  <p>After setup, this page closes and future users are created from Settings by an admin.</p>
                </div>
              </div>
              <div className="flex items-start gap-3 rounded-lg border border-slate-200 bg-slate-50 p-3">
                <Server className="mt-0.5 h-5 w-5 shrink-0 text-slate-700" />
                <div>
                  <p className="font-semibold text-slate-950">Version {__APP_VERSION__}</p>
                  <p>The browser, API server, and local data directory must be from the same deployment.</p>
                </div>
              </div>
            </div>
          </div>
        </section>

        <section className="card animate-scale-in">
          <div className="mb-6">
            <h2 className="text-lg font-semibold text-slate-950">Create your admin account to get started</h2>
            <p className="text-sm text-slate-600 mt-1">This will be the primary administrator account for your NeoNexus instance.</p>
          </div>

          {controlPlaneOffline && (
            <div className="mb-4 rounded-lg border border-red-200 bg-red-50 p-4 text-sm text-red-800">
              <div className="flex items-start gap-3">
                <AlertTriangle className="mt-0.5 h-5 w-5 shrink-0 text-red-700" />
                <div>
                  <p className="font-semibold text-red-950">Control plane API is offline</p>
                  <p className="mt-1 leading-6">
                    The web console loaded, but the setup API is not reachable. Start the full NeoNexus server and reload this page.
                  </p>
                </div>
              </div>
            </div>
          )}

          <FeedbackBanner error={error} suggestion={suggestion} code={code} />

          <form onSubmit={handleSubmit} className="space-y-4">
            <div>
              <label htmlFor="setup-username" className="block text-sm font-medium text-slate-700 mb-1">
                Username
              </label>
              <div className="relative">
                <User className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-slate-500" />
                <input
                  id="setup-username"
                  type="text"
                  value={username}
                  onChange={(e) => setUsername(e.target.value)}
                  className="input pl-10"
                  placeholder="Choose a username"
                  autoComplete="username"
                  required
                  autoFocus
                />
              </div>
            </div>

            <div>
              <label htmlFor="setup-password" className="block text-sm font-medium text-slate-700 mb-1">
                Password
              </label>
              <div className="relative">
                <Lock className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-slate-500" />
                <input
                  id="setup-password"
                  type="password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  className="input pl-10"
                  placeholder="Choose a password (min 8 chars)"
                  autoComplete="new-password"
                  required
                />
              </div>
            </div>

            <PasswordStrengthMeter password={password} />

            <div>
              <label htmlFor="setup-confirm-password" className="block text-sm font-medium text-slate-700 mb-1">
                Confirm Password
              </label>
              <div className="relative">
                <Lock className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-slate-500" />
                <input
                  id="setup-confirm-password"
                  type="password"
                  value={confirmPassword}
                  onChange={(e) => setConfirmPassword(e.target.value)}
                  className="input pl-10"
                  placeholder="Confirm your password"
                  autoComplete="new-password"
                  required
                />
              </div>
            </div>

            <button
              type="submit"
              disabled={isLoading}
              className="btn btn-primary w-full justify-center"
            >
              {isLoading ? (
                <>
                  <Loader2 className="w-4 h-4 animate-spin" />
                  Creating account...
                </>
              ) : (
                <>
                  <CheckCircle2 className="h-4 w-4" />
                  Create Admin Account
                </>
              )}
            </button>
          </form>
        </section>
      </div>
    </div>
  );
}
