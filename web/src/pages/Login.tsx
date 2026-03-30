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

  useEffect(() => {
    const checkSetupStatus = async () => {
      try {
        const response = await fetch('/api/auth/setup-status');
        if (response.ok) {
          const data = (await response.json()) as { needsSetup: boolean };
          if (data.needsSetup) {
            navigate('/setup', { replace: true });
            return;
          }
        }
      } catch {
        // If the check fails, proceed to login normally
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
    <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-slate-950 via-slate-900 to-blue-950 p-4">
      <div className="w-full max-w-md animate-scale-in">
        <div className="text-center mb-8">
          <div className="w-16 h-16 bg-blue-500/10 rounded-2xl flex items-center justify-center mx-auto mb-4">
            <Shield className="w-8 h-8 text-blue-500" />
          </div>
          <h1 className="text-2xl font-bold text-white"><span className="bg-gradient-to-r from-blue-400 to-cyan-400 bg-clip-text text-transparent">NeoNexus</span></h1>
          <p className="text-slate-400 mt-2">Node Manager</p>
        </div>

        {/* Default Credentials Warning */}
        <div className="mb-6 p-4 bg-yellow-500/10 border border-yellow-500/20 rounded-lg">
          <div className="flex items-start gap-3">
            <AlertTriangle className="w-5 h-5 text-yellow-500 shrink-0 mt-0.5" />
            <div className="text-sm text-yellow-400">
              <p className="font-medium mb-1">Default Credentials</p>
              <p className="opacity-90">Username: <strong>admin</strong></p>
              <p className="opacity-90">Password: <strong>admin</strong></p>
              <p className="mt-2 text-xs opacity-75">
                ⚠️ Please change the default password after first login in Settings → Change Password
              </p>
            </div>
          </div>
        </div>

        <div className="card">
          <FeedbackBanner error={error} suggestion={suggestion} code={code} />

          <form onSubmit={handleSubmit} className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-slate-300 mb-1">
                Username
              </label>
              <div className="relative">
                <User className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-slate-500" />
                <input
                  type="text"
                  value={username}
                  onChange={(e) => setUsername(e.target.value)}
                  className="input pl-10"
                  placeholder="Enter username"
                  required
                />
              </div>
            </div>

            <div>
              <label className="block text-sm font-medium text-slate-300 mb-1">
                Password
              </label>
              <div className="relative">
                <Lock className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-slate-500" />
                <input
                  type="password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  className="input pl-10"
                  placeholder="Enter password"
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
                  Signing in...
                </>
              ) : (
                'Sign In'
              )}
            </button>
          </form>
        </div>

        <div className="mt-6 text-center text-sm text-slate-500">
          <p>NeoNexus Node Manager v{__APP_VERSION__}</p>
          <p className="mt-1">Secure your node with strong credentials</p>
        </div>
      </div>
    </div>
  );
}
