import React, { useState } from 'react';
import {
  X,
  ExternalLink,
  Bookmark,
  Check,
  Sparkles,
  Layers,
  Highlighter,
  MessageSquare,
  Send,
  Trash2,
  BookOpen,
  GitMerge,
} from 'lucide-react';
import { apiFetch, formatRelativeTime } from '../api';
import { ArticleDetail, TeamBoard } from '../types';

interface Props {
  article: ArticleDetail | null;
  loading: boolean;
  boards: TeamBoard[];
  onClose: () => void;
  onRefreshArticle: (id: string) => void;
  onStateChanged: () => void;
}

export const ArticleReader: React.FC<Props> = ({
  article,
  loading,
  boards,
  onClose,
  onRefreshArticle,
  onStateChanged,
}) => {
  const [serif, setSerif] = useState(true);
  const [sepia, setSepia] = useState(false);
  const [summarizing, setSummarizing] = useState(false);
  const [showBoardsMenu, setShowBoardsMenu] = useState(false);
  const [highlightText, setHighlightText] = useState('');
  const [highlightColor, setHighlightColor] = useState('yellow');
  const [noteInput, setNoteInput] = useState('');
  const [activeTab, setActiveTab] = useState<'article' | 'collab'>('article');

  if (!article && !loading) return null;

  const handleAiSummary = async () => {
    if (!article) return;
    setSummarizing(true);
    try {
      await apiFetch(`/articles/${article.id}/ai-summary`, { method: 'POST' });
      onRefreshArticle(article.id);
    } finally {
      setSummarizing(false);
    }
  };

  const handleToggleState = async (patch: { is_read?: boolean; is_read_later?: boolean }) => {
    if (!article) return;
    await apiFetch(`/articles/${article.id}/state`, {
      method: 'PATCH',
      body: JSON.stringify(patch),
    });
    onRefreshArticle(article.id);
    onStateChanged();
  };

  const handleToggleBoard = async (boardId: string) => {
    if (!article) return;
    await apiFetch(`/boards/${boardId}/articles`, {
      method: 'POST',
      body: JSON.stringify({ article_id: article.id }),
    });
    onRefreshArticle(article.id);
    onStateChanged();
  };

  const handleCaptureSelection = () => {
    const sel = window.getSelection()?.toString().trim();
    if (sel) setHighlightText(sel);
  };

  const handleAddHighlight = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!article || !highlightText.trim()) return;
    await apiFetch(`/articles/${article.id}/highlights`, {
      method: 'POST',
      body: JSON.stringify({ selected_text: highlightText.trim(), color: highlightColor }),
    });
    setHighlightText('');
    onRefreshArticle(article.id);
  };

  const handleDeleteHighlight = async (hid: string) => {
    if (!article) return;
    await apiFetch(`/articles/${article.id}/highlights/${hid}`, { method: 'DELETE' });
    onRefreshArticle(article.id);
  };

  const handleAddNote = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!article || !noteInput.trim()) return;
    await apiFetch(`/articles/${article.id}/notes`, {
      method: 'POST',
      body: JSON.stringify({ content: noteInput.trim() }),
    });
    setNoteInput('');
    onRefreshArticle(article.id);
  };

  return (
    <div className="fixed inset-0 z-40 flex justify-end bg-slate-950/40 backdrop-blur-[2px]">
      <div
        className={`w-full max-w-3xl h-full flex flex-col shadow-2xl border-l border-slate-200 dark:border-slate-800 ${
          sepia
            ? 'bg-[#f8f3e6] text-[#2c251d]'
            : 'bg-white dark:bg-slate-950 text-slate-900 dark:text-slate-100'
        }`}
      >
        {/* Reader Top Bar */}
        <div className="px-5 py-3 border-b border-slate-200/80 dark:border-slate-800 flex items-center justify-between gap-2 shrink-0">
          <div className="flex items-center gap-1.5">
            <button
              onClick={onClose}
              className="p-2 rounded-xl hover:bg-slate-200/60 dark:hover:bg-slate-800 cursor-pointer"
              title="Close (Esc)"
            >
              <X className="w-4 h-4" />
            </button>
            <button
              onClick={() => setActiveTab('article')}
              className={`px-3 py-1.5 rounded-lg text-xs font-semibold cursor-pointer ${
                activeTab === 'article'
                  ? 'bg-emerald-500/15 text-emerald-700 dark:text-emerald-300'
                  : 'text-slate-500 hover:bg-slate-100 dark:hover:bg-slate-800'
              }`}
            >
              Reader
            </button>
            <button
              onClick={() => setActiveTab('collab')}
              className={`px-3 py-1.5 rounded-lg text-xs font-semibold flex items-center gap-1.5 cursor-pointer ${
                activeTab === 'collab'
                  ? 'bg-emerald-500/15 text-emerald-700 dark:text-emerald-300'
                  : 'text-slate-500 hover:bg-slate-100 dark:hover:bg-slate-800'
              }`}
            >
              <MessageSquare className="w-3.5 h-3.5" />
              Team Notes &amp; Highlights
              {article && (article.highlights.length + article.notes.length > 0) && (
                <span className="px-1.5 py-0.2 rounded-full bg-emerald-600 text-white text-[10px]">
                  {article.highlights.length + article.notes.length}
                </span>
              )}
            </button>
          </div>

          {article && (
            <div className="flex items-center gap-1.5">
              <button
                onClick={() => setSerif(!serif)}
                className="px-2 py-1 rounded-lg text-xs font-medium border border-slate-200 dark:border-slate-800 hover:bg-slate-100 dark:hover:bg-slate-800 cursor-pointer"
                title="Toggle Serif / Sans"
              >
                {serif ? 'Serif' : 'Sans'}
              </button>
              <button
                onClick={() => setSepia(!sepia)}
                className={`p-1.5 rounded-lg border border-slate-200 dark:border-slate-800 cursor-pointer ${
                  sepia ? 'bg-amber-200/70 text-amber-900' : 'text-slate-500 hover:bg-slate-100 dark:hover:bg-slate-800'
                }`}
                title="Toggle Sepia Reading Mode"
              >
                <BookOpen className="w-4 h-4" />
              </button>

              {/* Save to Team Board Dropdown */}
              <div className="relative">
                <button
                  onClick={() => setShowBoardsMenu(!showBoardsMenu)}
                  className={`px-2.5 py-1.5 rounded-lg text-xs font-semibold flex items-center gap-1 border cursor-pointer ${
                    article.board_ids.length > 0
                      ? 'bg-blue-500/15 text-blue-600 dark:text-blue-300 border-blue-500/30'
                      : 'border-slate-200 dark:border-slate-800 text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-800'
                  }`}
                >
                  <Layers className="w-3.5 h-3.5" />
                  Boards {article.board_ids.length > 0 ? `(${article.board_ids.length})` : ''}
                </button>
                {showBoardsMenu && (
                  <div className="absolute right-0 mt-1.5 w-56 rounded-xl bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 shadow-xl p-1.5 z-50">
                    <div className="px-2.5 py-1 text-[10px] font-bold uppercase tracking-wider text-slate-400">
                      Save to Team Board
                    </div>
                    {boards.map((b) => {
                      const saved = article.board_ids.includes(b.id);
                      return (
                        <button
                          key={b.id}
                          onClick={() => handleToggleBoard(b.id)}
                          className="w-full px-2.5 py-1.5 rounded-lg text-xs flex items-center justify-between hover:bg-slate-100 dark:hover:bg-slate-800 text-slate-800 dark:text-slate-200 cursor-pointer"
                        >
                          <span className="truncate">{b.name}</span>
                          {saved && <Check className="w-3.5 h-3.5 text-emerald-500" />}
                        </button>
                      );
                    })}
                  </div>
                )}
              </div>

              <button
                onClick={() => handleToggleState({ is_read_later: !article.is_read_later })}
                className={`p-1.5 rounded-lg border border-slate-200 dark:border-slate-800 cursor-pointer ${
                  article.is_read_later ? 'text-blue-500 bg-blue-500/10' : 'text-slate-500 hover:bg-slate-100 dark:hover:bg-slate-800'
                }`}
                title="Read Later"
              >
                <Bookmark className="w-4 h-4" fill={article.is_read_later ? 'currentColor' : 'none'} />
              </button>

              <button
                onClick={() => handleToggleState({ is_read: !article.is_read })}
                className="p-1.5 rounded-lg border border-slate-200 dark:border-slate-800 text-slate-500 hover:text-emerald-600 hover:bg-slate-100 dark:hover:bg-slate-800 cursor-pointer"
                title={article.is_read ? 'Mark Unread' : 'Mark Read'}
              >
                <Check className="w-4 h-4" />
              </button>

              <a
                href={article.url}
                target="_blank"
                rel="noopener noreferrer"
                className="px-2.5 py-1.5 rounded-lg bg-emerald-600 hover:bg-emerald-700 text-white text-xs font-semibold flex items-center gap-1"
              >
                Original
                <ExternalLink className="w-3.5 h-3.5" />
              </a>
            </div>
          )}
        </div>

        {/* Body */}
        {loading || !article ? (
          <div className="p-10 space-y-4">
            <div className="h-8 w-3/4 bg-slate-200 dark:bg-slate-800 rounded-xl animate-pulse" />
            <div className="h-4 w-1/3 bg-slate-200 dark:bg-slate-800 rounded-lg animate-pulse" />
            <div className="h-64 bg-slate-100 dark:bg-slate-900 rounded-2xl animate-pulse" />
          </div>
        ) : activeTab === 'article' ? (
          <div
            className="flex-1 overflow-y-auto px-7 py-6"
            onMouseUp={handleCaptureSelection}
          >
            {/* Meta */}
            <div className="flex items-center gap-2 flex-wrap text-xs text-slate-500 mb-2">
              <span className="font-semibold text-emerald-600 dark:text-emerald-400">
                {article.feed_title}
              </span>
              {article.author && (
                <>
                  <span>•</span>
                  <span>By {article.author}</span>
                </>
              )}
              <span>•</span>
              <span>{formatRelativeTime(article.published_at)}</span>
              <span>•</span>
              <span>{article.reading_time_mins} min read</span>
              <span className="px-2 py-0.5 rounded-full bg-violet-500/15 text-violet-700 dark:text-violet-300 font-bold">
                Leo Score: {article.priority_score}
              </span>
            </div>

            <h1 className={`text-2xl sm:text-3xl font-bold leading-tight mb-5 ${serif ? 'font-serif-reader' : ''}`}>
              {article.title}
            </h1>

            {/* Leo AI Summary Card */}
            <div className="mb-6 p-4 rounded-2xl bg-violet-500/10 border border-violet-500/20">
              <div className="flex items-center justify-between gap-2 mb-2">
                <div className="flex items-center gap-2 text-xs font-bold uppercase tracking-wider text-violet-700 dark:text-violet-300">
                  <Sparkles className="w-4 h-4" />
                  Leo Intelligence Brief
                  {article.ai_sentiment && (
                    <span className="px-2 py-0.5 rounded-full bg-violet-600 text-white text-[10px]">
                      {article.ai_sentiment}
                    </span>
                  )}
                </div>
                <button
                  onClick={handleAiSummary}
                  disabled={summarizing}
                  className="px-2.5 py-1 rounded-lg bg-violet-600 hover:bg-violet-700 disabled:opacity-50 text-white text-xs font-semibold cursor-pointer"
                >
                  {summarizing ? 'Analyzing...' : article.ai_summary ? 'Regenerate' : 'Generate AI Summary'}
                </button>
              </div>
              {article.ai_summary ? (
                <div className="text-sm whitespace-pre-line leading-relaxed text-slate-800 dark:text-slate-200">
                  {article.ai_summary}
                </div>
              ) : (
                <p className="text-xs text-slate-600 dark:text-slate-400">
                  Click &ldquo;Generate AI Summary&rdquo; for instant 3-bullet executive takeaways and tone classification (uses configured Gemini / OpenAI / Ollama or local extractive engine).
                </p>
              )}
            </div>

            {/* Duplicate / Related Sources Cluster */}
            {article.duplicates.length > 0 && (
              <div className="mb-6 p-4 rounded-2xl bg-amber-500/10 border border-amber-500/20">
                <div className="flex items-center gap-2 text-xs font-bold uppercase tracking-wider text-amber-700 dark:text-amber-300 mb-2">
                  <GitMerge className="w-4 h-4" />
                  Also Covered By ({article.duplicates.length} Deduplicated Sources)
                </div>
                <div className="space-y-1.5">
                  {article.duplicates.map((d) => (
                    <a
                      key={d.id}
                      href={d.url}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="flex items-center justify-between text-xs hover:underline text-slate-700 dark:text-slate-300"
                    >
                      <span className="truncate font-medium">{d.feed_title}: {d.title}</span>
                      <span className="text-slate-400 shrink-0 ml-2">{formatRelativeTime(d.published_at)}</span>
                    </a>
                  ))}
                </div>
              </div>
            )}

            {/* Quick Highlight Capture Bar if text selected */}
            {highlightText && (
              <form
                onSubmit={handleAddHighlight}
                className="mb-5 p-3 rounded-xl bg-amber-100/80 dark:bg-amber-950/50 border border-amber-300 dark:border-amber-800 flex items-center justify-between gap-3"
              >
                <div className="text-xs truncate flex-1">
                  <span className="font-bold">Save Highlight:</span> &ldquo;{highlightText}&rdquo;
                </div>
                <button
                  type="submit"
                  className="px-3 py-1 rounded-lg bg-amber-500 hover:bg-amber-600 text-white text-xs font-semibold shrink-0 cursor-pointer"
                >
                  Highlight
                </button>
              </form>
            )}

            {/* Article Sanitized HTML Body */}
            <div
              className={`article-prose ${serif ? 'font-serif-reader' : ''}`}
              dangerouslySetInnerHTML={{
                __html: article.content_html || `<p>${article.summary}</p>`,
              }}
            />
          </div>
        ) : (
          /* TEAM COLLABORATION: HIGHLIGHTS & NOTES */
          <div className="flex-1 overflow-y-auto px-7 py-6 space-y-6">
            <div>
              <h3 className="text-sm font-bold flex items-center gap-2 mb-3">
                <Highlighter className="w-4 h-4 text-amber-500" />
                Shared Highlights ({article.highlights.length})
              </h3>
              <form onSubmit={handleAddHighlight} className="flex gap-2 mb-4">
                <input
                  type="text"
                  value={highlightText}
                  onChange={(e) => setHighlightText(e.target.value)}
                  placeholder="Paste or select a quote from the article to highlight..."
                  className="flex-1 px-3 py-2 rounded-xl border border-slate-200 dark:border-slate-800 bg-slate-50 dark:bg-slate-900 text-sm"
                />
                <select
                  value={highlightColor}
                  onChange={(e) => setHighlightColor(e.target.value)}
                  className="px-2.5 py-2 rounded-xl border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 text-xs"
                >
                  <option value="yellow">Yellow</option>
                  <option value="green">Green</option>
                  <option value="blue">Blue</option>
                </select>
                <button
                  type="submit"
                  className="px-3.5 py-2 rounded-xl bg-amber-500 hover:bg-amber-600 text-white text-xs font-semibold cursor-pointer"
                >
                  Save
                </button>
              </form>

              <div className="space-y-2.5">
                {article.highlights.map((h) => (
                  <div
                    key={h.id}
                    className="p-3.5 rounded-xl bg-amber-50/70 dark:bg-amber-950/30 border-l-4 border-amber-400 flex items-start justify-between gap-2"
                  >
                    <div>
                      <p className="text-sm italic">&ldquo;{h.selected_text}&rdquo;</p>
                      <div className="text-[11px] text-slate-500 mt-1">
                        {h.user_name} • {formatRelativeTime(h.created_at)}
                      </div>
                    </div>
                    <button
                      onClick={() => handleDeleteHighlight(h.id)}
                      className="p-1 text-slate-400 hover:text-red-500 cursor-pointer"
                    >
                      <Trash2 className="w-3.5 h-3.5" />
                    </button>
                  </div>
                ))}
              </div>
            </div>

            <div className="pt-4 border-t border-slate-200 dark:border-slate-800">
              <h3 className="text-sm font-bold flex items-center gap-2 mb-3">
                <MessageSquare className="w-4 h-4 text-emerald-500" />
                Team Discussion Notes ({article.notes.length})
              </h3>
              <form onSubmit={handleAddNote} className="flex gap-2 mb-4">
                <input
                  type="text"
                  value={noteInput}
                  onChange={(e) => setNoteInput(e.target.value)}
                  placeholder="Add an enterprise team note or @mention..."
                  className="flex-1 px-3 py-2 rounded-xl border border-slate-200 dark:border-slate-800 bg-slate-50 dark:bg-slate-900 text-sm"
                />
                <button
                  type="submit"
                  className="px-3.5 py-2 rounded-xl bg-emerald-600 hover:bg-emerald-700 text-white text-xs font-semibold flex items-center gap-1 cursor-pointer"
                >
                  <Send className="w-3.5 h-3.5" />
                  Post
                </button>
              </form>

              <div className="space-y-3">
                {article.notes.map((n) => (
                  <div
                    key={n.id}
                    className="p-3.5 rounded-xl bg-slate-50 dark:bg-slate-900 border border-slate-200 dark:border-slate-800"
                  >
                    <div className="flex items-center justify-between text-xs mb-1">
                      <span className="font-bold text-slate-800 dark:text-slate-200">{n.user_name}</span>
                      <span className="text-slate-400">{formatRelativeTime(n.created_at)}</span>
                    </div>
                    <p className="text-sm text-slate-700 dark:text-slate-300">{n.content}</p>
                  </div>
                ))}
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
