import React, { useState } from 'react';
import { X, Rss, Search, Upload, Download, Plus, CheckCircle2 } from 'lucide-react';
import { apiFetch, getToken } from '../api';
import { DiscoveredFeed, Folder } from '../types';

interface Props {
  folders: Folder[];
  onClose: () => void;
  onChanged: () => void;
}

export const AddFeedModal: React.FC<Props> = ({ folders, onClose, onChanged }) => {
  const [tab, setTab] = useState<'subscribe' | 'opml'>('subscribe');
  const [urlInput, setUrlInput] = useState('');
  const [folderId, setFolderId] = useState<string>(folders[0]?.id || '');
  const [discovering, setDiscovering] = useState(false);
  const [discovered, setDiscovered] = useState<DiscoveredFeed[]>([]);
  const [subscribingUrl, setSubscribingUrl] = useState<string | null>(null);
  const [message, setMessage] = useState('');
  const [error, setError] = useState('');

  const handleDiscover = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!urlInput.trim()) return;
    setDiscovering(true);
    setError('');
    setMessage('');
    try {
      const res = await apiFetch<{ feeds: DiscoveredFeed[] }>('/feeds/discover', {
        method: 'POST',
        body: JSON.stringify({ url: urlInput.trim() }),
      });
      setDiscovered(res.feeds);
      if (res.feeds.length === 0) {
        setError('No RSS/Atom/JSON feeds discovered at that URL. You can still subscribe directly below.');
      }
    } catch (err: any) {
      setError(err.message);
    } finally {
      setDiscovering(false);
    }
  };

  const handleSubscribe = async (feedUrl: string, title?: string) => {
    setSubscribingUrl(feedUrl);
    setError('');
    try {
      const res = await apiFetch<{ new_articles: number }>('/feeds', {
        method: 'POST',
        body: JSON.stringify({
          feed_url: feedUrl,
          title: title || undefined,
          folder_id: folderId || null,
        }),
      });
      setMessage(`Subscribed! Fetched ${res.new_articles} articles.`);
      onChanged();
    } catch (err: any) {
      setError(err.message);
    } finally {
      setSubscribingUrl(null);
    }
  };

  const handleOpmlUpload = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;
    setError('');
    setMessage('');
    try {
      const xml = await file.text();
      const res = await apiFetch<{ folders_created: number; feeds_imported: number }>('/opml/import', {
        method: 'POST',
        body: JSON.stringify({ xml }),
      });
      setMessage(
        `OPML Imported: ${res.feeds_imported} feeds and ${res.folders_created} folders added! Background sync started.`
      );
      onChanged();
    } catch (err: any) {
      setError(err.message);
    }
  };

  const handleOpmlExport = async () => {
    const resp = await fetch('/api/opml/export', {
      headers: { Authorization: `Bearer ${getToken()}` },
    });
    const blob = await resp.blob();
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'pulsefeed-subscriptions.opml';
    a.click();
    URL.revokeObjectURL(url);
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/50 backdrop-blur-xs p-4">
      <div className="w-full max-w-lg rounded-2xl bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 shadow-2xl overflow-hidden">
        <div className="px-6 py-4 border-b border-slate-200 dark:border-slate-800 flex items-center justify-between">
          <div className="flex items-center gap-2 font-bold text-slate-900 dark:text-white">
            <Rss className="w-5 h-5 text-emerald-500" />
            Follow Unlimited Feeds &amp; OPML
          </div>
          <button onClick={onClose} className="p-1.5 rounded-lg hover:bg-slate-100 dark:hover:bg-slate-800 cursor-pointer">
            <X className="w-4 h-4 text-slate-500" />
          </button>
        </div>

        <div className="p-6">
          <div className="flex gap-2 mb-5">
            <button
              onClick={() => setTab('subscribe')}
              className={`flex-1 py-2 rounded-xl text-sm font-semibold cursor-pointer ${
                tab === 'subscribe'
                  ? 'bg-emerald-600 text-white'
                  : 'bg-slate-100 dark:bg-slate-800 text-slate-600 dark:text-slate-300'
              }`}
            >
              Add / Discover URL
            </button>
            <button
              onClick={() => setTab('opml')}
              className={`flex-1 py-2 rounded-xl text-sm font-semibold cursor-pointer ${
                tab === 'opml'
                  ? 'bg-emerald-600 text-white'
                  : 'bg-slate-100 dark:bg-slate-800 text-slate-600 dark:text-slate-300'
              }`}
            >
              OPML Import / Export
            </button>
          </div>

          {message && (
            <div className="mb-4 p-3 rounded-xl bg-emerald-50 dark:bg-emerald-950/40 border border-emerald-200 dark:border-emerald-900 text-emerald-700 dark:text-emerald-300 text-xs flex items-center gap-2">
              <CheckCircle2 className="w-4 h-4 shrink-0" />
              {message}
            </div>
          )}
          {error && (
            <div className="mb-4 p-3 rounded-xl bg-red-50 dark:bg-red-950/40 border border-red-200 dark:border-red-900 text-red-700 dark:text-red-300 text-xs">
              {error}
            </div>
          )}

          {tab === 'subscribe' ? (
            <div className="space-y-4">
              <div>
                <label className="block text-xs font-semibold text-slate-500 mb-1.5">
                  Target Folder
                </label>
                <select
                  value={folderId}
                  onChange={(e) => setFolderId(e.target.value)}
                  className="w-full px-3 py-2 rounded-xl border border-slate-200 dark:border-slate-700 bg-slate-50 dark:bg-slate-800 text-sm text-slate-900 dark:text-white"
                >
                  <option value="">Uncategorized</option>
                  {folders.map((f) => (
                    <option key={f.id} value={f.id}>{f.name}</option>
                  ))}
                </select>
              </div>

              <form onSubmit={handleDiscover} className="space-y-2">
                <label className="block text-xs font-semibold text-slate-500">
                  Website URL or Direct RSS / Atom / JSON Feed URL
                </label>
                <div className="flex gap-2">
                  <input
                    type="text"
                    required
                    value={urlInput}
                    onChange={(e) => setUrlInput(e.target.value)}
                    placeholder="https://techcrunch.com or https://site.com/feed.xml"
                    className="flex-1 px-3.5 py-2.5 rounded-xl border border-slate-200 dark:border-slate-700 bg-slate-50 dark:bg-slate-800 text-sm text-slate-900 dark:text-white"
                  />
                  <button
                    type="submit"
                    disabled={discovering}
                    className="px-4 py-2.5 rounded-xl bg-emerald-600 hover:bg-emerald-700 text-white text-sm font-semibold flex items-center gap-1.5 cursor-pointer disabled:opacity-50"
                  >
                    <Search className="w-4 h-4" />
                    {discovering ? 'Scanning...' : 'Find'}
                  </button>
                </div>
              </form>

              {discovered.length > 0 && (
                <div className="space-y-2 pt-2">
                  <div className="text-xs font-bold uppercase tracking-wider text-slate-400">
                    Discovered Feeds
                  </div>
                  {discovered.map((df) => (
                    <div
                      key={df.feed_url}
                      className="p-3 rounded-xl border border-slate-200 dark:border-slate-800 flex items-center justify-between gap-3"
                    >
                      <div className="min-w-0">
                        <div className="text-sm font-semibold text-slate-900 dark:text-white truncate">
                          {df.title}
                        </div>
                        <div className="text-xs text-slate-500 truncate">{df.feed_url}</div>
                      </div>
                      <button
                        onClick={() => handleSubscribe(df.feed_url, df.title)}
                        disabled={subscribingUrl === df.feed_url}
                        className="px-3 py-1.5 rounded-lg bg-emerald-600 hover:bg-emerald-700 text-white text-xs font-semibold flex items-center gap-1 shrink-0 cursor-pointer"
                      >
                        <Plus className="w-3.5 h-3.5" />
                        {subscribingUrl === df.feed_url ? 'Adding...' : 'Follow'}
                      </button>
                    </div>
                  ))}
                </div>
              )}
            </div>
          ) : (
            <div className="space-y-4">
              <div className="p-5 rounded-2xl border-2 border-dashed border-slate-300 dark:border-slate-700 text-center">
                <Upload className="w-8 h-8 text-emerald-500 mx-auto mb-2" />
                <div className="text-sm font-semibold text-slate-900 dark:text-white">
                  Import Unlimited Feeds from Feedly / Inoreader (.opml / .xml)
                </div>
                <p className="text-xs text-slate-500 mt-1 mb-4">
                  Folders and all subscriptions are imported automatically in one click.
                </p>
                <label className="inline-flex items-center gap-2 px-4 py-2 rounded-xl bg-emerald-600 hover:bg-emerald-700 text-white text-xs font-semibold cursor-pointer">
                  Choose OPML File
                  <input type="file" accept=".opml,.xml" onChange={handleOpmlUpload} className="hidden" />
                </label>
              </div>

              <button
                onClick={handleOpmlExport}
                className="w-full py-2.5 rounded-xl border border-slate-200 dark:border-slate-700 hover:bg-slate-50 dark:hover:bg-slate-800 text-sm font-semibold text-slate-700 dark:text-slate-200 flex items-center justify-center gap-2 cursor-pointer"
              >
                <Download className="w-4 h-4" />
                Export All Subscriptions as OPML 2.0
              </button>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
