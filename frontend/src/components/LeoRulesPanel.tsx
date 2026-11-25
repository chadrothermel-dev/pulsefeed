import React, { useEffect, useState } from 'react';
import { Sparkles, Plus, Trash2, Play, Power, VolumeX, Tag, Layers, Bell } from 'lucide-react';
import { apiFetch } from '../api';
import { LeoRule, TeamBoard, WebhookEndpoint } from '../types';

interface Props {
  boards: TeamBoard[];
  onChanged: () => void;
}

export const LeoRulesPanel: React.FC<Props> = ({ boards, onChanged }) => {
  const [rules, setRules] = useState<LeoRule[]>([]);
  const [webhooks, setWebhooks] = useState<WebhookEndpoint[]>([]);
  const [name, setName] = useState('');
  const [ruleType, setRuleType] = useState('priority');
  const [keywords, setKeywords] = useState('');
  const [matchField, setMatchField] = useState('any');
  const [scoreDelta, setScoreDelta] = useState(25);
  const [targetTag, setTargetTag] = useState('');
  const [targetBoardId, setTargetBoardId] = useState('');
  const [targetWebhookId, setTargetWebhookId] = useState('');
  const [running, setRunning] = useState(false);
  const [statusMsg, setStatusMsg] = useState('');

  const load = async () => {
    const [r, w] = await Promise.all([
      apiFetch<{ rules: LeoRule[] }>('/leo/rules'),
      apiFetch<{ webhooks: WebhookEndpoint[] }>('/webhooks'),
    ]);
    setRules(r.rules);
    setWebhooks(w.webhooks);
  };

  useEffect(() => { load(); }, []);

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!name.trim() || !keywords.trim()) return;
    await apiFetch('/leo/rules', {
      method: 'POST',
      body: JSON.stringify({
        name: name.trim(),
        rule_type: ruleType,
        keywords: keywords.trim(),
        match_field: matchField,
        score_delta: Number(scoreDelta),
        target_tag: targetTag.trim() || null,
        target_board_id: targetBoardId || null,
        target_webhook_id: targetWebhookId || null,
      }),
    });
    setName('');
    setKeywords('');
    setTargetTag('');
    setTargetBoardId('');
    setTargetWebhookId('');
    await load();
    onChanged();
  };

  const handleToggle = async (id: string) => {
    await apiFetch(`/leo/rules/${id}/toggle`, { method: 'POST' });
    await load();
  };

  const handleDelete = async (id: string) => {
    await apiFetch(`/leo/rules/${id}`, { method: 'DELETE' });
    await load();
  };

  const handleRerun = async () => {
    setRunning(true);
    setStatusMsg('');
    try {
      const res = await apiFetch<{ rescored: number }>('/leo/rules/rerun', { method: 'POST' });
      setStatusMsg(`Leo re-scored ${res.rescored} recent articles using your active rules!`);
      onChanged();
    } finally {
      setRunning(false);
    }
  };

  return (
    <div className="flex-1 h-screen overflow-y-auto bg-white dark:bg-slate-950 p-6 lg:p-8">
      <div className="max-w-5xl mx-auto space-y-6">
        <div className="flex flex-wrap items-center justify-between gap-4">
          <div>
            <h1 className="text-2xl font-bold text-slate-900 dark:text-white flex items-center gap-2.5">
              <Sparkles className="w-6 h-6 text-violet-500" />
              Leo AI Intelligence &amp; Automation Rules
            </h1>
            <p className="text-sm text-slate-500 mt-1">
              Prioritize high-signal stories, mute noise, auto-tag, auto-curate to Team Boards, and trigger Slack/Teams/Webhook alerts.
            </p>
          </div>
          <button
            onClick={handleRerun}
            disabled={running}
            className="px-4 py-2 rounded-xl bg-violet-600 hover:bg-violet-700 disabled:opacity-50 text-white text-sm font-semibold flex items-center gap-2 shadow-sm cursor-pointer"
          >
            <Play className="w-4 h-4" />
            {running ? 'Re-scoring...' : 'Apply Rules to Existing Articles'}
          </button>
        </div>

        {statusMsg && (
          <div className="p-3.5 rounded-xl bg-emerald-50 dark:bg-emerald-950/40 border border-emerald-200 dark:border-emerald-900 text-emerald-700 dark:text-emerald-300 text-sm">
            {statusMsg}
          </div>
        )}

        {/* Create Rule Form */}
        <form
          onSubmit={handleCreate}
          className="p-5 rounded-2xl border border-slate-200 dark:border-slate-800 bg-slate-50/70 dark:bg-slate-900/60 space-y-4"
        >
          <div className="text-sm font-bold text-slate-900 dark:text-white">Create New Leo Rule</div>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
            <input
              type="text"
              required
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="Rule Name (e.g. Zero-Day Threats)"
              className="px-3 py-2 rounded-xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 text-sm text-slate-900 dark:text-white"
            />
            <select
              value={ruleType}
              onChange={(e) => setRuleType(e.target.value)}
              className="px-3 py-2 rounded-xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 text-sm text-slate-900 dark:text-white"
            >
              <option value="priority">Priority Boost</option>
              <option value="mute">Mute Filter (Hide Noise)</option>
              <option value="auto_tag">Auto-Tag</option>
              <option value="auto_board">Auto-Save to Team Board</option>
              <option value="alert">Real-Time Webhook Alert</option>
            </select>
            <select
              value={matchField}
              onChange={(e) => setMatchField(e.target.value)}
              className="px-3 py-2 rounded-xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 text-sm text-slate-900 dark:text-white"
            >
              <option value="any">Match: Title + Body + Author</option>
              <option value="title">Match: Title Only</option>
              <option value="content">Match: Body Content</option>
              <option value="author">Match: Author</option>
            </select>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-4 gap-3">
            <input
              type="text"
              required
              value={keywords}
              onChange={(e) => setKeywords(e.target.value)}
              placeholder="Keywords (comma-separated or /regex/)"
              className="md:col-span-2 px-3 py-2 rounded-xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 text-sm text-slate-900 dark:text-white"
            />
            <input
              type="text"
              value={targetTag}
              onChange={(e) => setTargetTag(e.target.value)}
              placeholder="Auto-tag (optional)"
              className="px-3 py-2 rounded-xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 text-sm text-slate-900 dark:text-white"
            />
            <input
              type="number"
              value={scoreDelta}
              onChange={(e) => setScoreDelta(Number(e.target.value))}
              placeholder="Score delta"
              className="px-3 py-2 rounded-xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 text-sm text-slate-900 dark:text-white"
            />
          </div>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-3 items-center">
            <select
              value={targetBoardId}
              onChange={(e) => setTargetBoardId(e.target.value)}
              className="px-3 py-2 rounded-xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 text-sm text-slate-900 dark:text-white"
            >
              <option value="">Auto-save to Board (optional)</option>
              {boards.map((b) => (
                <option key={b.id} value={b.id}>{b.name}</option>
              ))}
            </select>

            <select
              value={targetWebhookId}
              onChange={(e) => setTargetWebhookId(e.target.value)}
              className="px-3 py-2 rounded-xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 text-sm text-slate-900 dark:text-white"
            >
              <option value="">Trigger Webhook Alert (optional)</option>
              {webhooks.map((w) => (
                <option key={w.id} value={w.id}>{w.name} ({w.kind})</option>
              ))}
            </select>

            <button
              type="submit"
              className="py-2 px-4 rounded-xl bg-emerald-600 hover:bg-emerald-700 text-white text-sm font-semibold flex items-center justify-center gap-1.5 cursor-pointer"
            >
              <Plus className="w-4 h-4" />
              Add Leo Rule
            </button>
          </div>
        </form>

        {/* Active Rules List */}
        <div className="space-y-3">
          {rules.map((r) => (
            <div
              key={r.id}
              className={`p-4 rounded-2xl border flex items-center justify-between gap-4 ${
                r.is_enabled
                  ? 'border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900'
                  : 'border-slate-200/50 dark:border-slate-800/50 opacity-50 bg-slate-50 dark:bg-slate-900/30'
              }`}
            >
              <div className="min-w-0">
                <div className="flex items-center gap-2 flex-wrap">
                  <span className="font-bold text-slate-900 dark:text-white">{r.name}</span>
                  <span className="px-2 py-0.5 rounded-full text-[11px] font-bold uppercase bg-violet-500/15 text-violet-600 dark:text-violet-300 flex items-center gap-1">
                    {r.rule_type === 'mute' ? <VolumeX className="w-3 h-3" /> :
                     r.rule_type === 'auto_tag' ? <Tag className="w-3 h-3" /> :
                     r.rule_type === 'auto_board' ? <Layers className="w-3 h-3" /> :
                     r.rule_type === 'alert' ? <Bell className="w-3 h-3" /> :
                     <Sparkles className="w-3 h-3" />}
                    {r.rule_type}
                  </span>
                  <span className="text-xs text-slate-400">Matches: {r.match_count}</span>
                </div>
                <div className="text-xs text-slate-500 mt-1 font-mono truncate">
                  [{r.match_field}] {r.keywords}
                </div>
              </div>

              <div className="flex items-center gap-2 shrink-0">
                <button
                  onClick={() => handleToggle(r.id)}
                  className={`p-2 rounded-lg border cursor-pointer ${
                    r.is_enabled
                      ? 'border-emerald-500/30 text-emerald-600 bg-emerald-500/10'
                      : 'border-slate-200 dark:border-slate-700 text-slate-400'
                  }`}
                  title="Enable/Disable Rule"
                >
                  <Power className="w-4 h-4" />
                </button>
                <button
                  onClick={() => handleDelete(r.id)}
                  className="p-2 rounded-lg border border-slate-200 dark:border-slate-800 text-slate-400 hover:text-red-500 cursor-pointer"
                  title="Delete Rule"
                >
                  <Trash2 className="w-4 h-4" />
                </button>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};
