import React, { useEffect, useState } from 'react';
import {
  X,
  Rss,
  Search,
  Upload,
  Download,
  Plus,
  CheckCircle2,
  Check,
  Sparkles,
  Globe,
  Compass,
} from 'lucide-react';
import { apiFetch, getToken } from '../api';
import { DiscoveredFeed, FeedSearchResult, Folder } from '../types';

interface Props {
  folders: Folder[];
  onClose: () => void;
  onChanged: () => void;
}

export const AddFeedModal: React.FC<Props> = ({ folders, onClose, onChanged }) => {
  const [tab, setTab] = useState<'search' | 'url' | 'opml'>('search');
  const [folderId, setFolderId] = useState<string>(folders[0]?.id || '');

  // Topic Search state
  const [topicQuery, setTopicQuery] = useState('');
  const [activeCategory, setActiveCategory] = useState('All');
  const [categories, setCategories] = useState<string[]>([
    'All', 'Tech', 'Programming', 'Cybersecurity', 'AI & ML',
    'Cloud & Infra', 'World News', 'Finance', 'Science', 'Sports', 'Gaming',
  ]);
  const [searchResults, setSearchResults] = useState<FeedSearchResult[]>([]);
  const [searching, setSearching] = useState(false);

  // Direct URL Discovery state
  const [urlInput, setUrlInput] = useState('');
  const [discovering, setDiscovering] = useState(false);
  const [discovered, setDiscovered] = useState<DiscoveredFeed[]>([]);

  const [subscribingUrl, setSubscribingUrl] = useState<string | null>(null);
  const [message, setMessage] = useState('');
  const [error, setError] = useState('');

  const runTopicSearch = async (q: string, cat: string) => {
    setSearching(true);
    setError('');
    try {
      const params = new URLSearchParams();
      if (q.trim()) params.set('q', q.trim());
      if (cat && cat !== 'All') params.set('category', cat);
      const res = await apiFetch<{ categories: string[]; results: FeedSearchResult[] }>(
        `/feeds/search?${params.toString()}`
      );
      if (res.categories?.length) setCategories(res.categories);
      setSearchResults(res.results);
    } catch (err: any) {
      setError(err.message);
    } finally {
      setSearching(false);
    }
  };

  useEffect(() => {
    if (tab !== 'search') return;
    const timer = setTimeout(() => {
      runTopicSearch(topicQuery, activeCategory);
    }, topicQuery ? 280 : 0);
    return () => clearTimeout(timer);
  }, [tab, topicQuery, activeCategory]);

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
        setError('No RSS/Atom/JSON feeds discovered at that URL.');
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
    setMessage('');
    try {
      const res = await apiFetch<{ new_articles: number }>('/feeds', {
        method: 'POST',
        body: JSON.stringify({
          feed_url: feedUrl,
          title: title || undefined,
          folder_id: folderId || null,
        }),
      });
      setMessage(`Followed "${title || feedUrl}" — synced ${res.new_articles} articles!`);
      setSearchResults((prev) =>
        prev.map((r) =>
          r.feed_url.toLowerCase() === feedUrl.toLowerCase()
            ? { ...r, already_subscribed: true }
            : r
        )
      );
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
    const token = getToken();
    const resp = await fetch('/api/opml/export', {
      headers: token ? { Authorization: `Bearer ${token}` } : {},
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
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/55 backdrop-blur-xs p-4">
      <div className="w-full max-w-2xl max-h-[88vh] flex flex-col rounded-2xl bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 shadow-2xl overflow-hidden">
        {/* Header */}
        <div className="px-6 py-4 border-b border-slate-200 dark:border-slate-800 flex items-center justify-between shrink-0">
          <div className="flex items-center gap-2 font-bold text-slate-900 dark:text-white">
            <Compass className="w-5 h-5 text-emerald-500" />
            Discover &amp; Follow Feeds
          </div>
          <button onClick={onClose} className="p-1.5 rounded-lg hover:bg-slate-100 dark:hover:bg-slate-800 cursor-pointer">
            <X className="w-4 h-4 text-slate-500" />
          </button>
        </div>

        {/* Tabs + Target Folder */}
        <div className="px-6 pt-4 pb-3 border-b border-slate-100 dark:border-slate-800/80 space-y-3 shrink-0">
          <div className="flex flex-wrap items-center justify-between gap-3">
            <div className="flex gap-1.5 bg-slate-100 dark:bg-slate-800 p-1 rounded-xl">
              <button
                onClick={() => setTab('search')}
                className={`px-3 py-1.5 rounded-lg text-xs font-semibold flex items-center gap-1.5 cursor-pointer transition ${
                  tab === 'search'
                    ? 'bg-emerald-600 text-white shadow-xs'
                    : 'text-slate-600 dark:text-slate-300'
                }`}
              >
                <Search className="w-3.5 h-3.5" />
                Search by Topic
              </button>
              <button
                onClick={() => setTab('url')}
                className={`px-3 py-1.5 rounded-lg text-xs font-semibold flex items-center gap-1.5 cursor-pointer transition ${
                  tab === 'url'
                    ? 'bg-emerald-600 text-white shadow-xs'
                    : 'text-slate-600 dark:text-slate-300'
                }`}
              >
                <Globe className="w-3.5 h-3.5" />
                Direct URL
              </button>
              <button
                onClick={() => setTab('opml')}
                className={`px-3 py-1.5 rounded-lg text-xs font-semibold flex items-center gap-1.5 cursor-pointer transition ${
                  tab === 'opml'
                    ? 'bg-emerald-600 text-white shadow-xs'
                    : 'text-slate-600 dark:text-slate-300'
                }`}
              >
                <Upload className="w-3.5 h-3.5" />
                OPML
              </button>
            </div>

            {tab !== 'opml' && (
              <div className="flex items-center gap-2">
                <span className="text-xs font-semibold text-slate-500">Save to:</span>
                <select
                  value={folderId}
                  onChange={(e) => setFolderId(e.target.value)}
                  className="px-2.5 py-1.5 rounded-lg border border-slate-200 dark:border-slate-700 bg-slate-50 dark:bg-slate-800 text-xs font-medium text-slate-900 dark:text-white"
                >
                  <option value="">Uncategorized</option>
                  {folders.map((f) => (
                    <option key={f.id} value={f.id}>{f.name}</option>
                  ))}
                </select>
              </div>
            )}
          </div>

          {message && (
            <div className="p-2.5 rounded-xl bg-emerald-50 dark:bg-emerald-950/40 border border-emerald-200 dark:border-emerald-900 text-emerald-700 dark:text-emerald-300 text-xs flex items-center gap-2">
              <CheckCircle2 className="w-4 h-4 shrink-0" />
              {message}
            </div>
          )}
          {error && (
            <div className="p-2.5 rounded-xl bg-red-50 dark:bg-red-950/40 border border-red-200 dark:border-red-900 text-red-700 dark:text-red-300 text-xs">
              {error}
            </div>
          )}
        </div>

        {/* Body */}
        <div className="flex-1 overflow-y-auto p-6">
          {tab === 'search' ? (
            <div className="space-y-4">
              {/* Topic Search Bar */}
              <div className="relative">
                <Search className="w-4 h-4 text-slate-400 absolute left-3.5 top-1/2 -translate-y-1/2" />
                <input
                  type="text"
                  autoFocus
                  value={topicQuery}
                  onChange={(e) => setTopicQuery(e.target.value)}
                  placeholder="Search any topic, interest, or publication (e.g. cybersecurity, AI, gardening, NFL, Rust)..."
                  className="w-full pl-10 pr-4 py-2.5 rounded-xl border border-slate-200 dark:border-slate-700 bg-slate-50 dark:bg-slate-800 text-sm text-slate-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-emerald-500"
                />
              </div>

              {/* Category Browse Pills */}
              <div className="flex flex-wrap gap-1.5">
                {categories.map((cat) => (
                  <button
                    key={cat}
                    onClick={() => setActiveCategory(cat)}
                    className={`px-2.5 py-1 rounded-full text-xs font-semibold transition cursor-pointer ${
                      activeCategory === cat
                        ? 'bg-emerald-600 text-white'
                        : 'bg-slate-100 dark:bg-slate-800 text-slate-600 dark:text-slate-300 hover:bg-slate-200 dark:hover:bg-slate-700'
                    }`}
                  >
                    {cat}
                  </button>
                ))}
              </div>

              {/* Results */}
              {searching ? (
                <div className="space-y-2.5">
                  {[1, 2, 3, 4].map((i) => (
                    <div key={i} className="h-16 rounded-xl bg-slate-100 dark:bg-slate-800/70 animate-pulse" />
                  ))}
                </div>
              ) : searchResults.length === 0 ? (
                <div className="text-center py-10 text-sm text-slate-500">
                  No feeds found. Try another topic or paste a website URL in the &ldquo;Direct URL&rdquo; tab.
                </div>
              ) : (
                <div className="space-y-2.5">
                  {searchResults.map((item) => (
                    <div
                      key={item.feed_url}
                      className="p-3.5 rounded-xl border border-slate-200/90 dark:border-slate-800 hover:border-emerald-500/40 bg-white dark:bg-slate-900/60 flex items-center justify-between gap-3 transition"
                    >
                      <div className="flex items-start gap-3 min-w-0">
                        <div className="w-9 h-9 rounded-lg bg-slate-100 dark:bg-slate-800 flex items-center justify-center shrink-0 overflow-hidden mt-0.5">
                          {item.icon_url ? (
                            <img
                              src={item.icon_url}
                              alt=""
                              className="w-6 h-6 object-contain rounded"
                              onError={(e) => { (e.target as HTMLImageElement).style.display = 'none'; }}
                            />
                          ) : (
                            <Rss className="w-4 h-4 text-emerald-500" />
                          )}
                        </div>
                        <div className="min-w-0">
                          <div className="flex items-center gap-2 flex-wrap">
                            <span className="text-sm font-bold text-slate-900 dark:text-white truncate">
                              {item.title}
                            </span>
                            <span className="text-[10px] px-2 py-0.5 rounded-full font-semibold bg-slate-100 dark:bg-slate-800 text-slate-600 dark:text-slate-300">
                              {item.category}
                            </span>
                            {item.source === 'topic_feed' && (
                              <span className="text-[10px] px-2 py-0.5 rounded-full font-semibold bg-violet-500/15 text-violet-600 dark:text-violet-300 flex items-center gap-1">
                                <Sparkles className="w-2.5 h-2.5" /> Live Topic
                              </span>
                            )}
                            {item.subscribers && item.subscribers > 0 && (
                              <span className="text-[10px] text-slate-400">
                                {item.subscribers >= 1000
                                  ? `${(item.subscribers / 1000).toFixed(1)}k readers`
                                  : `${item.subscribers} readers`}
                              </span>
                            )}
                          </div>
                          {item.description && (
                            <p className="text-xs text-slate-500 dark:text-slate-400 line-clamp-2 mt-0.5">
                              {item.description}
                            </p>
                          )}
                        </div>
                      </div>

                      {item.already_subscribed ? (
                        <span className="px-3 py-1.5 rounded-lg bg-emerald-500/15 text-emerald-600 dark:text-emerald-300 text-xs font-semibold flex items-center gap-1 shrink-0">
                          <Check className="w-3.5 h-3.5" />
                          Following
                        </span>
                      ) : (
                        <button
                          onClick={() => handleSubscribe(item.feed_url, item.title)}
                          disabled={subscribingUrl === item.feed_url}
                          className="px-3 py-1.5 rounded-lg bg-emerald-600 hover:bg-emerald-700 disabled:opacity-50 text-white text-xs font-semibold flex items-center gap-1 shrink-0 cursor-pointer"
                        >
                          <Plus className="w-3.5 h-3.5" />
                          {subscribingUrl === item.feed_url ? 'Adding...' : 'Follow'}
                        </button>
                      )}
                    </div>
                  ))}
                </div>
              )}
            </div>
          ) : tab === 'url' ? (
            <div className="space-y-4">
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
