import React, { useEffect, useState } from 'react';
import {
  Shield,
  Activity,
  Users,
  Webhook,
  Key,
  Settings,
  FileText,
  Plus,
  Trash2,
  Send,
  Copy,
  CheckCircle2,
  Database,
  Rss,
  AlertTriangle,
} from 'lucide-react';
import { apiFetch, formatRelativeTime } from '../api';
import {
  ApiKeyRecord,
  AuditLogEntry,
  SystemSettings,
  TelemetryStats,
  User,
  WebhookEndpoint,
} from '../types';

interface Props {
  edition: 'personal' | 'enterprise';
  currentUser: User;
}

export const EnterpriseAdminPanel: React.FC<Props> = ({ edition, currentUser }) => {
  const isPersonal = edition === 'personal';
  const isAdmin = currentUser.role === 'admin';
  const [tab, setTab] = useState<'telemetry' | 'users' | 'webhooks' | 'apikeys' | 'settings' | 'audit'>('telemetry');

  const [telemetry, setTelemetry] = useState<TelemetryStats | null>(null);
  const [users, setUsers] = useState<User[]>([]);
  const [webhooks, setWebhooks] = useState<WebhookEndpoint[]>([]);
  const [apiKeys, setApiKeys] = useState<ApiKeyRecord[]>([]);
  const [logs, setLogs] = useState<AuditLogEntry[]>([]);
  const [settings, setSettings] = useState<SystemSettings | null>(null);
  const [banner, setBanner] = useState('');

  // Form states
  const [newUserEmail, setNewUserEmail] = useState('');
  const [newUserName, setNewUserName] = useState('');
  const [newUserPass, setNewUserPass] = useState('');
  const [newUserRole, setNewUserRole] = useState('member');

  const [whName, setWhName] = useState('');
  const [whKind, setWhKind] = useState('slack');
  const [whUrl, setWhUrl] = useState('');
  const [whSecret, setWhSecret] = useState('');

  const [keyName, setKeyName] = useState('');
  const [freshRawKey, setFreshRawKey] = useState<string | null>(null);

  const [llmApiKeyInput, setLlmApiKeyInput] = useState('');

  const loadAll = async () => {
    const [tel, wh, keys] = await Promise.all([
      apiFetch<TelemetryStats>('/admin/telemetry'),
      apiFetch<{ webhooks: WebhookEndpoint[] }>('/webhooks'),
      apiFetch<{ api_keys: ApiKeyRecord[] }>('/admin/api-keys'),
    ]);
    setTelemetry(tel);
    setWebhooks(wh.webhooks);
    setApiKeys(keys.api_keys);

    if (isAdmin) {
      const [u, l, s] = await Promise.all([
        apiFetch<{ users: User[] }>('/admin/users'),
        apiFetch<{ logs: AuditLogEntry[] }>('/admin/audit-logs'),
        apiFetch<SystemSettings>('/admin/settings'),
      ]);
      setUsers(u.users);
      setLogs(l.logs);
      setSettings(s);
    }
  };

  useEffect(() => {
    loadAll().catch(() => {});
  }, []);

  const handleCreateUser = async (e: React.FormEvent) => {
    e.preventDefault();
    await apiFetch('/admin/users', {
      method: 'POST',
      body: JSON.stringify({
        email: newUserEmail,
        name: newUserName,
        password: newUserPass,
        role: newUserRole,
      }),
    });
    setNewUserEmail('');
    setNewUserName('');
    setNewUserPass('');
    setBanner('Enterprise user created.');
    loadAll();
  };

  const handleUpdateUser = async (id: string, patch: { role?: string; is_active?: boolean }) => {
    await apiFetch(`/admin/users/${id}`, {
      method: 'PATCH',
      body: JSON.stringify(patch),
    });
    loadAll();
  };

  const handleCreateWebhook = async (e: React.FormEvent) => {
    e.preventDefault();
    await apiFetch('/webhooks', {
      method: 'POST',
      body: JSON.stringify({ name: whName, kind: whKind, url: whUrl, secret: whSecret || null }),
    });
    setWhName('');
    setWhUrl('');
    setWhSecret('');
    setBanner('Webhook endpoint registered.');
    loadAll();
  };

  const handleTestWebhook = async (id: string) => {
    await apiFetch(`/webhooks/${id}/test`, { method: 'POST' });
    setBanner('Test alert dispatched — check Last Status.');
    loadAll();
  };

  const handleDeleteWebhook = async (id: string) => {
    await apiFetch(`/webhooks/${id}`, { method: 'DELETE' });
    loadAll();
  };

  const handleCreateApiKey = async (e: React.FormEvent) => {
    e.preventDefault();
    const res = await apiFetch<{ raw_key: string }>('/admin/api-keys', {
      method: 'POST',
      body: JSON.stringify({ name: keyName }),
    });
    setKeyName('');
    setFreshRawKey(res.raw_key);
    loadAll();
  };

  const handleRevokeKey = async (id: string) => {
    await apiFetch(`/admin/api-keys/${id}`, { method: 'DELETE' });
    loadAll();
  };

  const handleSaveSettings = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!settings) return;
    await apiFetch('/admin/settings', {
      method: 'PATCH',
      body: JSON.stringify({
        ...settings,
        llm_api_key: llmApiKeyInput || undefined,
      }),
    });
    setLlmApiKeyInput('');
    setBanner('Enterprise LLM, SSO & Engine settings saved.');
    loadAll();
  };

  const tabs = [
    { id: 'telemetry', label: 'Overview & Health', icon: Activity, adminOnly: false, enterpriseOnly: false },
    { id: 'users', label: 'Users & RBAC', icon: Users, adminOnly: true, enterpriseOnly: true },
    { id: 'webhooks', label: 'Webhooks & Alerts', icon: Webhook, adminOnly: false, enterpriseOnly: false },
    { id: 'apikeys', label: 'API Keys', icon: Key, adminOnly: false, enterpriseOnly: false },
    { id: 'settings', label: isPersonal ? 'AI / LLM Settings' : 'LLM & SSO', icon: Settings, adminOnly: true, enterpriseOnly: false },
    { id: 'audit', label: 'Audit Logs', icon: FileText, adminOnly: true, enterpriseOnly: true },
  ] as const;

  return (
    <div className="flex-1 h-screen overflow-y-auto bg-white dark:bg-slate-950 p-6 lg:p-8">
      <div className="max-w-5xl mx-auto space-y-6">
        <div>
          <h1 className="text-2xl font-bold text-slate-900 dark:text-white flex items-center gap-2.5">
            <Shield className="w-6 h-6 text-emerald-500" />
            {isPersonal ? 'Settings, AI & System Health' : 'Enterprise Administration & Integrations'}
          </h1>
          <p className="text-sm text-slate-500 mt-1">
            {isPersonal
              ? 'Monitor feed health, configure optional Gemini/OpenAI/Ollama AI summaries, webhooks, and local API keys.'
              : 'Manage RBAC users, Slack/Teams webhooks, SIEM API keys, Hybrid LLM providers, OIDC SSO, and audit logs.'}
          </p>
        </div>

        {/* Sub-Nav Tabs */}
        <div className="flex flex-wrap gap-2 border-b border-slate-200 dark:border-slate-800 pb-3">
          {tabs
            .filter((t) => (!t.adminOnly || isAdmin) && (!t.enterpriseOnly || !isPersonal))
            .map((t) => {
              const Icon = t.icon;
              return (
                <button
                  key={t.id}
                  onClick={() => { setTab(t.id); setBanner(''); }}
                  className={`px-3.5 py-2 rounded-xl text-xs font-semibold flex items-center gap-2 transition cursor-pointer ${
                    tab === t.id
                      ? 'bg-emerald-600 text-white shadow-sm'
                      : 'bg-slate-100 dark:bg-slate-900 text-slate-600 dark:text-slate-300 hover:bg-slate-200 dark:hover:bg-slate-800'
                  }`}
                >
                  <Icon className="w-4 h-4" />
                  {t.label}
                </button>
              );
            })}
        </div>

        {banner && (
          <div className="p-3 rounded-xl bg-emerald-50 dark:bg-emerald-950/40 border border-emerald-200 dark:border-emerald-900 text-emerald-700 dark:text-emerald-300 text-xs flex items-center gap-2">
            <CheckCircle2 className="w-4 h-4" />
            {banner}
          </div>
        )}

        {/* TELEMETRY TAB */}
        {tab === 'telemetry' && telemetry && (
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            {[
              { label: 'Total Feeds (Unlimited)', val: telemetry.total_feeds, icon: Rss, color: 'text-emerald-500' },
              { label: 'Indexed Articles (FTS5)', val: telemetry.total_articles, icon: FileText, color: 'text-blue-500' },
              { label: 'Failing Feeds', val: telemetry.failing_feeds, icon: AlertTriangle, color: 'text-amber-500' },
              { label: 'SQLite DB Size', val: `${(telemetry.db_size_bytes / 1024 / 1024).toFixed(2)} MB`, icon: Database, color: 'text-violet-500' },
              { label: 'Workspace Users', val: telemetry.total_users, icon: Users, color: 'text-cyan-500' },
              { label: isPersonal ? 'Saved Boards' : 'Team Boards', val: telemetry.total_boards, icon: Shield, color: 'text-emerald-500' },
              { label: 'Active Leo Rules', val: telemetry.total_rules, icon: Activity, color: 'text-violet-500' },
              { label: 'Webhooks', val: telemetry.total_webhooks, icon: Webhook, color: 'text-pink-500' },
            ].map((card) => {
              const Icon = card.icon;
              return (
                <div
                  key={card.label}
                  className="p-5 rounded-2xl border border-slate-200 dark:border-slate-800 bg-slate-50/70 dark:bg-slate-900/60"
                >
                  <Icon className={`w-5 h-5 ${card.color} mb-3`} />
                  <div className="text-2xl font-bold text-slate-900 dark:text-white">{card.val}</div>
                  <div className="text-xs text-slate-500 mt-1">{card.label}</div>
                </div>
              );
            })}
          </div>
        )}

        {/* USERS & RBAC TAB */}
        {tab === 'users' && isAdmin && (
          <div className="space-y-5">
            <form
              onSubmit={handleCreateUser}
              className="p-4 rounded-2xl border border-slate-200 dark:border-slate-800 bg-slate-50 dark:bg-slate-900/60 grid grid-cols-1 md:grid-cols-5 gap-3"
            >
              <input
                type="text"
                required
                value={newUserName}
                onChange={(e) => setNewUserName(e.target.value)}
                placeholder="Full Name"
                className="px-3 py-2 rounded-xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 text-sm text-slate-900 dark:text-white"
              />
              <input
                type="email"
                required
                value={newUserEmail}
                onChange={(e) => setNewUserEmail(e.target.value)}
                placeholder="Email"
                className="px-3 py-2 rounded-xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 text-sm text-slate-900 dark:text-white"
              />
              <input
                type="password"
                required
                value={newUserPass}
                onChange={(e) => setNewUserPass(e.target.value)}
                placeholder="Password"
                className="px-3 py-2 rounded-xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 text-sm text-slate-900 dark:text-white"
              />
              <select
                value={newUserRole}
                onChange={(e) => setNewUserRole(e.target.value)}
                className="px-3 py-2 rounded-xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 text-sm text-slate-900 dark:text-white"
              >
                <option value="admin">Admin</option>
                <option value="editor">Editor</option>
                <option value="member">Member</option>
                <option value="viewer">Viewer (Read-Only)</option>
              </select>
              <button
                type="submit"
                className="px-4 py-2 rounded-xl bg-emerald-600 hover:bg-emerald-700 text-white text-sm font-semibold flex items-center justify-center gap-1 cursor-pointer"
              >
                <Plus className="w-4 h-4" /> Add User
              </button>
            </form>

            <div className="border border-slate-200 dark:border-slate-800 rounded-2xl overflow-hidden">
              <table className="w-full text-sm">
                <thead className="bg-slate-50 dark:bg-slate-900 text-left text-xs uppercase text-slate-500">
                  <tr>
                    <th className="px-4 py-3">User</th>
                    <th className="px-4 py-3">Role</th>
                    <th className="px-4 py-3">Status</th>
                    <th className="px-4 py-3 text-right">Actions</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-slate-100 dark:divide-slate-800">
                  {users.map((u) => (
                    <tr key={u.id}>
                      <td className="px-4 py-3">
                        <div className="font-semibold text-slate-900 dark:text-white">{u.name}</div>
                        <div className="text-xs text-slate-500">{u.email}</div>
                      </td>
                      <td className="px-4 py-3">
                        <select
                          value={u.role}
                          onChange={(e) => handleUpdateUser(u.id, { role: e.target.value })}
                          className="px-2 py-1 rounded-lg border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 text-xs font-semibold"
                        >
                          <option value="admin">admin</option>
                          <option value="editor">editor</option>
                          <option value="member">member</option>
                          <option value="viewer">viewer</option>
                        </select>
                      </td>
                      <td className="px-4 py-3">
                        <span
                          className={`px-2 py-0.5 rounded-full text-xs font-semibold ${
                            u.is_active
                              ? 'bg-emerald-500/15 text-emerald-600 dark:text-emerald-300'
                              : 'bg-red-500/15 text-red-600'
                          }`}
                        >
                          {u.is_active ? 'Active' : 'Disabled'}
                        </span>
                      </td>
                      <td className="px-4 py-3 text-right">
                        <button
                          onClick={() => handleUpdateUser(u.id, { is_active: !u.is_active })}
                          className="text-xs font-semibold text-slate-500 hover:text-slate-900 dark:hover:text-white cursor-pointer"
                        >
                          {u.is_active ? 'Disable' : 'Enable'}
                        </button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        )}

        {/* WEBHOOKS TAB */}
        {tab === 'webhooks' && (
          <div className="space-y-5">
            <form
              onSubmit={handleCreateWebhook}
              className="p-4 rounded-2xl border border-slate-200 dark:border-slate-800 bg-slate-50 dark:bg-slate-900/60 grid grid-cols-1 md:grid-cols-5 gap-3"
            >
              <input
                type="text"
                required
                value={whName}
                onChange={(e) => setWhName(e.target.value)}
                placeholder="Name (e.g. SOC Slack)"
                className="px-3 py-2 rounded-xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 text-sm text-slate-900 dark:text-white"
              />
              <select
                value={whKind}
                onChange={(e) => setWhKind(e.target.value)}
                className="px-3 py-2 rounded-xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 text-sm text-slate-900 dark:text-white"
              >
                <option value="slack">Slack Incoming Webhook</option>
                <option value="teams">Microsoft Teams</option>
                <option value="generic">Generic JSON Webhook</option>
              </select>
              <input
                type="url"
                required
                value={whUrl}
                onChange={(e) => setWhUrl(e.target.value)}
                placeholder="https://hooks.slack.com/services/..."
                className="md:col-span-2 px-3 py-2 rounded-xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 text-sm text-slate-900 dark:text-white"
              />
              <button
                type="submit"
                className="px-4 py-2 rounded-xl bg-emerald-600 hover:bg-emerald-700 text-white text-sm font-semibold cursor-pointer"
              >
                Add Webhook
              </button>
            </form>

            <div className="space-y-2.5">
              {webhooks.map((w) => (
                <div
                  key={w.id}
                  className="p-4 rounded-2xl border border-slate-200 dark:border-slate-800 flex items-center justify-between gap-4"
                >
                  <div className="min-w-0">
                    <div className="flex items-center gap-2">
                      <span className="font-bold text-slate-900 dark:text-white">{w.name}</span>
                      <span className="px-2 py-0.5 rounded-full bg-blue-500/15 text-blue-600 dark:text-blue-300 text-[11px] font-bold uppercase">
                        {w.kind}
                      </span>
                      {w.last_status && (
                        <span className="text-xs text-slate-400">Last: {w.last_status}</span>
                      )}
                    </div>
                    <div className="text-xs text-slate-500 font-mono truncate mt-0.5">{w.url}</div>
                  </div>
                  <div className="flex items-center gap-2 shrink-0">
                    <button
                      onClick={() => handleTestWebhook(w.id)}
                      className="px-3 py-1.5 rounded-lg border border-slate-200 dark:border-slate-700 text-xs font-semibold flex items-center gap-1 hover:bg-slate-100 dark:hover:bg-slate-800 cursor-pointer"
                    >
                      <Send className="w-3.5 h-3.5" /> Test
                    </button>
                    <button
                      onClick={() => handleDeleteWebhook(w.id)}
                      className="p-1.5 rounded-lg text-slate-400 hover:text-red-500 cursor-pointer"
                    >
                      <Trash2 className="w-4 h-4" />
                    </button>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* API KEYS TAB */}
        {tab === 'apikeys' && (
          <div className="space-y-5">
            <form onSubmit={handleCreateApiKey} className="flex gap-3">
              <input
                type="text"
                required
                value={keyName}
                onChange={(e) => setKeyName(e.target.value)}
                placeholder="Key Name (e.g. SIEM Ingestion / Python Script)"
                className="flex-1 px-3.5 py-2.5 rounded-xl border border-slate-200 dark:border-slate-700 bg-slate-50 dark:bg-slate-900 text-sm text-slate-900 dark:text-white"
              />
              <button
                type="submit"
                className="px-4 py-2.5 rounded-xl bg-emerald-600 hover:bg-emerald-700 text-white text-sm font-semibold cursor-pointer"
              >
                Generate API Key
              </button>
            </form>

            {freshRawKey && (
              <div className="p-4 rounded-2xl bg-amber-50 dark:bg-amber-950/40 border border-amber-300 dark:border-amber-800 flex items-center justify-between gap-3">
                <div className="text-xs">
                  <div className="font-bold text-amber-800 dark:text-amber-300">
                    Copy your API key now (it won&apos;t be shown again):
                  </div>
                  <code className="font-mono text-sm text-slate-900 dark:text-white break-all">{freshRawKey}</code>
                </div>
                <button
                  onClick={() => { navigator.clipboard.writeText(freshRawKey); setBanner('API key copied!'); }}
                  className="px-3 py-1.5 rounded-lg bg-amber-500 text-white text-xs font-semibold flex items-center gap-1 shrink-0 cursor-pointer"
                >
                  <Copy className="w-3.5 h-3.5" /> Copy
                </button>
              </div>
            )}

            <div className="space-y-2">
              {apiKeys.map((k) => (
                <div
                  key={k.id}
                  className="p-4 rounded-2xl border border-slate-200 dark:border-slate-800 flex items-center justify-between"
                >
                  <div>
                    <div className="font-semibold text-sm text-slate-900 dark:text-white">{k.name}</div>
                    <div className="text-xs text-slate-500 font-mono">
                      {k.key_prefix}•••••••• • Owner: {k.user_email} • Last used: {k.last_used_at ? formatRelativeTime(k.last_used_at) : 'Never'}
                    </div>
                  </div>
                  <button
                    onClick={() => handleRevokeKey(k.id)}
                    className="px-3 py-1.5 rounded-lg text-xs font-semibold text-red-500 hover:bg-red-50 dark:hover:bg-red-950/40 cursor-pointer"
                  >
                    Revoke
                  </button>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* SETTINGS: LLM & SSO */}
        {tab === 'settings' && isAdmin && settings && (
          <form onSubmit={handleSaveSettings} className="space-y-6">
            <div className="p-5 rounded-2xl border border-slate-200 dark:border-slate-800 space-y-4">
              <h3 className="font-bold text-slate-900 dark:text-white">
                Hybrid AI Provider (Optional LLM for Summaries &amp; Sentiment)
              </h3>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div>
                  <label className="block text-xs font-semibold text-slate-500 mb-1">Provider</label>
                  <select
                    value={settings.llm_provider}
                    onChange={(e) => setSettings({ ...settings, llm_provider: e.target.value })}
                    className="w-full px-3 py-2 rounded-xl border border-slate-200 dark:border-slate-700 bg-slate-50 dark:bg-slate-800 text-sm text-slate-900 dark:text-white"
                  >
                    <option value="none">Local Extractive Only (Zero API Key)</option>
                    <option value="gemini">Google Gemini API</option>
                    <option value="openai">OpenAI / Compatible API</option>
                    <option value="ollama">Local Ollama Server</option>
                  </select>
                </div>
                <div>
                  <label className="block text-xs font-semibold text-slate-500 mb-1">Model Name</label>
                  <input
                    type="text"
                    value={settings.llm_model}
                    onChange={(e) => setSettings({ ...settings, llm_model: e.target.value })}
                    placeholder="gemini-2.5-flash / gpt-4o-mini / llama3.2"
                    className="w-full px-3 py-2 rounded-xl border border-slate-200 dark:border-slate-700 bg-slate-50 dark:bg-slate-800 text-sm text-slate-900 dark:text-white"
                  />
                </div>
                <div>
                  <label className="block text-xs font-semibold text-slate-500 mb-1">
                    API Key {settings.llm_api_key_set && '(Currently Set)'}
                  </label>
                  <input
                    type="password"
                    value={llmApiKeyInput}
                    onChange={(e) => setLlmApiKeyInput(e.target.value)}
                    placeholder="Enter new API key to update..."
                    className="w-full px-3 py-2 rounded-xl border border-slate-200 dark:border-slate-700 bg-slate-50 dark:bg-slate-800 text-sm text-slate-900 dark:text-white"
                  />
                </div>
                <div>
                  <label className="block text-xs font-semibold text-slate-500 mb-1">Custom Base URL (Ollama / Proxy)</label>
                  <input
                    type="text"
                    value={settings.llm_base_url}
                    onChange={(e) => setSettings({ ...settings, llm_base_url: e.target.value })}
                    placeholder="http://127.0.0.1:11434"
                    className="w-full px-3 py-2 rounded-xl border border-slate-200 dark:border-slate-700 bg-slate-50 dark:bg-slate-800 text-sm text-slate-900 dark:text-white"
                  />
                </div>
              </div>
            </div>

            {!isPersonal && (
              <div className="p-5 rounded-2xl border border-slate-200 dark:border-slate-800 space-y-4">
                <div className="flex items-center justify-between">
                  <h3 className="font-bold text-slate-900 dark:text-white">
                    Enterprise SSO / OIDC Configuration
                  </h3>
                  <label className="flex items-center gap-2 text-xs font-semibold">
                    <input
                      type="checkbox"
                      checked={settings.sso_enabled}
                      onChange={(e) => setSettings({ ...settings, sso_enabled: e.target.checked })}
                    />
                    Enable SSO Button
                  </label>
                </div>
                <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                  <input
                    type="text"
                    value={settings.sso_provider_name}
                    onChange={(e) => setSettings({ ...settings, sso_provider_name: e.target.value })}
                    placeholder="Microsoft Entra ID / Okta"
                    className="px-3 py-2 rounded-xl border border-slate-200 dark:border-slate-700 bg-slate-50 dark:bg-slate-800 text-sm text-slate-900 dark:text-white"
                  />
                  <input
                    type="text"
                    value={settings.sso_issuer_url}
                    onChange={(e) => setSettings({ ...settings, sso_issuer_url: e.target.value })}
                    placeholder="https://login.microsoftonline.com/.../v2.0"
                    className="px-3 py-2 rounded-xl border border-slate-200 dark:border-slate-700 bg-slate-50 dark:bg-slate-800 text-sm text-slate-900 dark:text-white"
                  />
                  <input
                    type="text"
                    value={settings.sso_client_id}
                    onChange={(e) => setSettings({ ...settings, sso_client_id: e.target.value })}
                    placeholder="OIDC Client ID"
                    className="px-3 py-2 rounded-xl border border-slate-200 dark:border-slate-700 bg-slate-50 dark:bg-slate-800 text-sm text-slate-900 dark:text-white"
                  />
                </div>
              </div>
            )}

            <button
              type="submit"
              className="px-5 py-2.5 rounded-xl bg-emerald-600 hover:bg-emerald-700 text-white text-sm font-semibold cursor-pointer"
            >
              {isPersonal ? 'Save Settings' : 'Save Enterprise Settings'}
            </button>
          </form>
        )}

        {/* AUDIT LOGS TAB */}
        {tab === 'audit' && isAdmin && (
          <div className="border border-slate-200 dark:border-slate-800 rounded-2xl overflow-hidden">
            <table className="w-full text-xs">
              <thead className="bg-slate-50 dark:bg-slate-900 text-left uppercase text-slate-500">
                <tr>
                  <th className="px-4 py-3">Time</th>
                  <th className="px-4 py-3">Actor</th>
                  <th className="px-4 py-3">Action</th>
                  <th className="px-4 py-3">Details</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-100 dark:divide-slate-800">
                {logs.map((l) => (
                  <tr key={l.id}>
                    <td className="px-4 py-2.5 text-slate-400 whitespace-nowrap">
                      {formatRelativeTime(l.created_at)}
                    </td>
                    <td className="px-4 py-2.5 font-medium text-slate-700 dark:text-slate-300">
                      {l.user_email || 'system'}
                    </td>
                    <td className="px-4 py-2.5 font-mono text-emerald-600 dark:text-emerald-400">
                      {l.action}
                    </td>
                    <td className="px-4 py-2.5 text-slate-600 dark:text-slate-400">{l.details}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </div>
  );
};
