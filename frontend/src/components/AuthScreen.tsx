import React, { useEffect, useState } from 'react';
import { Rss, ShieldCheck, Sparkles, Lock, Mail, User as UserIcon, ArrowRight } from 'lucide-react';
import { apiFetch, setToken } from '../api';
import { User } from '../types';

interface Props {
  onSuccess: (user: User) => void;
}

export const AuthScreen: React.FC<Props> = ({ onSuccess }) => {
  const [mode, setMode] = useState<'login' | 'register'>('login');
  const [email, setEmail] = useState('admin@pulsefeed.local');
  const [password, setPassword] = useState('admin123');
  const [name, setName] = useState('');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);
  const [providers, setProviders] = useState<{ id: string; name: string }[]>([]);

  useEffect(() => {
    const params = new URLSearchParams(window.location.search);
    const oauthErr = params.get('oauth_error');
    if (oauthErr) {
      setError(oauthErr);
      window.history.replaceState({}, '', window.location.pathname);
    }
    apiFetch<{ providers: { id: string; name: string }[] }>('/auth/providers')
      .then((res) => setProviders(res.providers || []))
      .catch(() => {});
  }, []);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');
    setLoading(true);
    try {
      const endpoint = mode === 'login' ? '/auth/login' : '/auth/register';
      const body = mode === 'login' ? { email, password } : { email, password, name };
      const res = await apiFetch<{ token: string; user: User }>(endpoint, {
        method: 'POST',
        body: JSON.stringify(body),
      });
      setToken(res.token);
      onSuccess(res.user);
    } catch (err: any) {
      setError(err.message || 'Authentication failed');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="min-h-screen bg-slate-50 dark:bg-slate-950 flex items-center justify-center p-4">
      <div className="w-full max-w-md">
        <div className="text-center mb-8">
          <div className="inline-flex items-center justify-center w-14 h-14 rounded-2xl bg-emerald-600 text-white shadow-lg shadow-emerald-600/25 mb-4">
            <Rss className="w-7 h-7" />
          </div>
          <h1 className="text-3xl font-bold tracking-tight text-slate-900 dark:text-white">
            PulseFeed
          </h1>
          <p className="text-slate-500 dark:text-slate-400 mt-1.5 text-sm">
            High-efficiency Enterprise RSS &amp; Intelligence Engine
          </p>
        </div>

        <div className="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-2xl shadow-xl p-7">
          <div className="flex rounded-xl bg-slate-100 dark:bg-slate-800 p-1 mb-6">
            <button
              type="button"
              onClick={() => { setMode('login'); setError(''); }}
              className={`flex-1 py-2 text-sm font-medium rounded-lg transition ${
                mode === 'login'
                  ? 'bg-white dark:bg-slate-700 text-slate-900 dark:text-white shadow-sm'
                  : 'text-slate-600 dark:text-slate-400'
              }`}
            >
              Sign In
            </button>
            <button
              type="button"
              onClick={() => { setMode('register'); setEmail(''); setPassword(''); setError(''); }}
              className={`flex-1 py-2 text-sm font-medium rounded-lg transition ${
                mode === 'register'
                  ? 'bg-white dark:bg-slate-700 text-slate-900 dark:text-white shadow-sm'
                  : 'text-slate-600 dark:text-slate-400'
              }`}
            >
              Create Account
            </button>
          </div>

          {error && (
            <div className="mb-4 p-3 rounded-xl bg-red-50 dark:bg-red-950/40 border border-red-200 dark:border-red-900 text-red-700 dark:text-red-300 text-sm">
              {error}
            </div>
          )}

          <form onSubmit={handleSubmit} className="space-y-4">
            {mode === 'register' && (
              <div>
                <label className="block text-xs font-semibold uppercase tracking-wider text-slate-500 mb-1.5">
                  Full Name
                </label>
                <div className="relative">
                  <UserIcon className="w-4 h-4 text-slate-400 absolute left-3.5 top-1/2 -translate-y-1/2" />
                  <input
                    type="text"
                    required
                    value={name}
                    onChange={(e) => setName(e.target.value)}
                    placeholder="Alex Rivera"
                    className="w-full pl-10 pr-3.5 py-2.5 rounded-xl border border-slate-200 dark:border-slate-700 bg-slate-50 dark:bg-slate-800 text-slate-900 dark:text-white text-sm focus:outline-none focus:ring-2 focus:ring-emerald-500"
                  />
                </div>
              </div>
            )}

            <div>
              <label className="block text-xs font-semibold uppercase tracking-wider text-slate-500 mb-1.5">
                Work Email
              </label>
              <div className="relative">
                <Mail className="w-4 h-4 text-slate-400 absolute left-3.5 top-1/2 -translate-y-1/2" />
                <input
                  type="email"
                  required
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  placeholder="you@company.com"
                  className="w-full pl-10 pr-3.5 py-2.5 rounded-xl border border-slate-200 dark:border-slate-700 bg-slate-50 dark:bg-slate-800 text-slate-900 dark:text-white text-sm focus:outline-none focus:ring-2 focus:ring-emerald-500"
                />
              </div>
            </div>

            <div>
              <label className="block text-xs font-semibold uppercase tracking-wider text-slate-500 mb-1.5">
                Password
              </label>
              <div className="relative">
                <Lock className="w-4 h-4 text-slate-400 absolute left-3.5 top-1/2 -translate-y-1/2" />
                <input
                  type="password"
                  required
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  placeholder="••••••••"
                  className="w-full pl-10 pr-3.5 py-2.5 rounded-xl border border-slate-200 dark:border-slate-700 bg-slate-50 dark:bg-slate-800 text-slate-900 dark:text-white text-sm focus:outline-none focus:ring-2 focus:ring-emerald-500"
                />
              </div>
            </div>

            <button
              type="submit"
              disabled={loading}
              className="w-full py-2.5 px-4 rounded-xl bg-emerald-600 hover:bg-emerald-700 disabled:opacity-50 text-white font-semibold text-sm flex items-center justify-center gap-2 shadow-md shadow-emerald-600/20 transition cursor-pointer"
            >
              {loading ? 'Signing in...' : mode === 'login' ? 'Sign In to Workspace' : 'Create Account'}
              <ArrowRight className="w-4 h-4" />
            </button>
          </form>

          {providers.length > 0 && (
            <div className="mt-5 pt-5 border-t border-slate-200 dark:border-slate-800 space-y-2.5">
              <div className="text-[11px] font-semibold uppercase tracking-wider text-center text-slate-400 mb-2">
                Or continue with Identity Provider
              </div>
              {providers.map((p) => {
                const badgeColor =
                  p.id === 'google'
                    ? 'bg-red-500/15 text-red-600 dark:text-red-400'
                    : p.id === 'entra'
                    ? 'bg-sky-500/15 text-sky-600 dark:text-sky-400'
                    : 'bg-blue-600/15 text-blue-600 dark:text-blue-400';
                const monogram = p.id === 'google' ? 'G' : p.id === 'entra' ? 'M' : 'f';
                return (
                  <a
                    key={p.id}
                    href={`/api/auth/oauth/${p.id}/start`}
                    className="w-full py-2.5 px-4 rounded-xl border border-slate-200 dark:border-slate-700 hover:bg-slate-50 dark:hover:bg-slate-800 text-slate-700 dark:text-slate-200 text-sm font-semibold flex items-center justify-center gap-2.5 transition"
                  >
                    <span className={`w-5 h-5 rounded-md text-xs font-extrabold flex items-center justify-center ${badgeColor}`}>
                      {monogram}
                    </span>
                    Continue with {p.name}
                  </a>
                );
              })}
            </div>
          )}

          {mode === 'login' && (
            <div className="mt-5 p-3 rounded-xl bg-emerald-50/70 dark:bg-emerald-950/30 border border-emerald-200/70 dark:border-emerald-900/50 text-xs text-emerald-800 dark:text-emerald-300 flex items-start gap-2">
              <Sparkles className="w-4 h-4 shrink-0 mt-0.5 text-emerald-600" />
              <div>
                <span className="font-semibold">Default Admin:</span>{' '}
                <code className="font-mono">admin@pulsefeed.local</code> /{' '}
                <code className="font-mono">admin123</code>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
