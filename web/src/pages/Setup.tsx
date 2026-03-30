import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { Activity, Lock, User, Loader2 } from 'lucide-react';
import { api, ApiRequestError } from '../utils/api';
import { FeedbackBanner } from '../components/FeedbackBanner';

interface SetupResponse {
  token: string;
  user: { id: number; username: string; role: string };
}

export default function Setup() {
  const navigate = useNavigate();
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [error, setError] = useState('');
  const [suggestion, setSuggestion] = useState('');
  const [code, setCode] = useState('');
  const [isLoading, setIsLoading] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');
    setSuggestion('');
    setCode('');

    if (!username || !password || !confirmPassword) {
      setError('All fields are required');
      return;
    }

    if (password.length < 6) {
      setError('Password must be at least 6 characters');
      return;
    }

    if (password !== confirmPassword) {
      setError('Passwords do not match');
      return;
    }

    setIsLoading(true);

    try {
      const data = await api.post<SetupResponse>('/api/auth/setup', { username, password });
      localStorage.setItem('token', data.token);
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

  return (
    <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-slate-950 via-slate-900 to-blue-950 p-4">
      <div className="w-full max-w-md animate-scale-in">
        <div className="text-center mb-8">
          <div className="w-16 h-16 bg-blue-500/10 rounded-2xl flex items-center justify-center mx-auto mb-4">
            <Activity className="w-8 h-8 text-blue-500" />
          </div>
          <h1 className="text-2xl font-bold text-white">
            <span className="bg-gradient-to-r from-blue-400 to-cyan-400 bg-clip-text text-transparent">NeoNexus</span>
          </h1>
          <p className="text-slate-400 mt-2">Welcome to NeoNexus</p>
        </div>

        <div className="card">
          <div className="mb-6">
            <h2 className="text-lg font-semibold text-white">Create your admin account to get started</h2>
            <p className="text-sm text-slate-400 mt-1">This will be the primary administrator account for your NeoNexus instance.</p>
          </div>

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
                  placeholder="Choose a username"
                  required
                  autoFocus
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
                  placeholder="Choose a password (min 6 chars)"
                  required
                />
              </div>
            </div>

            <div>
              <label className="block text-sm font-medium text-slate-300 mb-1">
                Confirm Password
              </label>
              <div className="relative">
                <Lock className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-slate-500" />
                <input
                  type="password"
                  value={confirmPassword}
                  onChange={(e) => setConfirmPassword(e.target.value)}
                  className="input pl-10"
                  placeholder="Confirm your password"
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
                'Create Admin Account'
              )}
            </button>
          </form>
        </div>

        <div className="mt-6 text-center text-sm text-slate-500">
          <p>NeoNexus Node Manager v{__APP_VERSION__}</p>
        </div>
      </div>
    </div>
  );
}
