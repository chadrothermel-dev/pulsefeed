import React from 'react';
import {
  Search,
  RefreshCw,
  CheckCheck,
  LayoutList,
  LayoutGrid,
  AlignJustify,
  Sparkles,
  Bookmark,
  ExternalLink,
  Copy,
  MessageSquare,
  Highlighter,
  Check,
  Layers,
  Filter,
  Trash2,
} from 'lucide-react';
import { ArticleListItem, LayoutMode, TeamBoard } from '../types';
import { formatRelativeTime } from '../api';

interface Props {
  title: string;
  subtitle?: string;
  board?: TeamBoard;
  articles: ArticleListItem[];
  selectedId: string | null;
  loading: boolean;
  refreshing: boolean;
  searchQuery: string;
  onSearchChange: (q: string) => void;
  unreadOnly: boolean;
  onToggleUnreadOnly: () => void;
  dedup: boolean;
  onToggleDedup: () => void;
  sort: 'newest' | 'priority' | 'oldest';
  onChangeSort: (s: 'newest' | 'priority' | 'oldest') => void;
  layout: LayoutMode;
  onChangeLayout: (l: LayoutMode) => void;
  onRefresh: () => void;
  onMarkAllRead: () => void;
  deleteLabel?: string;
  onDeleteCurrent?: () => void;
  onSelectArticle: (id: string) => void;
  onToggleRead: (article: ArticleListItem, e: React.MouseEvent) => void;
  onToggleReadLater: (article: ArticleListItem, e: React.MouseEvent) => void;
}

export const ArticleList: React.FC<Props> = ({
  title,
  subtitle,
  board,
  articles,
  selectedId,
  loading,
  refreshing,
  searchQuery,
  onSearchChange,
  unreadOnly,
  onToggleUnreadOnly,
  dedup,
  onToggleDedup,
  sort,
  onChangeSort,
  layout,
  onChangeLayout,
  onRefresh,
  onMarkAllRead,
  deleteLabel,
  onDeleteCurrent,
  onSelectArticle,
  onToggleRead,
  onToggleReadLater,
}) => {
  const copyBoardRss = () => {
    if (!board) return;
    const url = `${window.location.origin}/api/public/boards/${board.public_token}/rss`;
    navigator.clipboard.writeText(url);
    alert(`Public Board RSS URL copied:\n${url}`);
  };

  return (
    <div className="flex-1 flex flex-col min-w-0 h-screen bg-white dark:bg-slate-950">
      {/* Top Toolbar */}
      <header className="px-6 py-3.5 border-b border-slate-200 dark:border-slate-800 flex flex-wrap items-center justify-between gap-3 bg-white/90 dark:bg-slate-950/90 backdrop-blur sticky top-0 z-10">
        <div className="min-w-0">
          <div className="flex items-center gap-2.5">
            <h1 className="text-xl font-bold text-slate-900 dark:text-white truncate">{title}</h1>
            {board && board.visibility === 'public' && (
              <button
                onClick={copyBoardRss}
                className="text-xs px-2 py-0.5 rounded-full bg-emerald-500/15 text-emerald-600 dark:text-emerald-300 font-medium flex items-center gap-1 hover:bg-emerald-500/25 cursor-pointer"
              >
                <Copy className="w-3 h-3" />
                Public RSS
              </button>
            )}
            {onDeleteCurrent && deleteLabel && (
              <button
                onClick={onDeleteCurrent}
                className="text-xs px-2.5 py-1 rounded-lg border border-red-200 dark:border-red-900/60 bg-red-50/70 dark:bg-red-950/30 text-red-600 dark:text-red-400 font-semibold flex items-center gap-1 hover:bg-red-100 dark:hover:bg-red-950/60 transition cursor-pointer shrink-0"
              >
                <Trash2 className="w-3.5 h-3.5" />
                {deleteLabel}
              </button>
            )}
          </div>
          {subtitle && <p className="text-xs text-slate-500 truncate mt-0.5">{subtitle}</p>}
        </div>

        <div className="flex items-center gap-2 flex-wrap">
          {/* FTS5 Search */}
          <div className="relative">
            <Search className="w-4 h-4 text-slate-400 absolute left-3 top-1/2 -translate-y-1/2" />
            <input
              id="pulsefeed-search-input"
              type="text"
              value={searchQuery}
              onChange={(e) => onSearchChange(e.target.value)}
              placeholder="Search articles (/) ..."
              className="w-56 lg:w-64 pl-9 pr-3 py-1.5 rounded-xl border border-slate-200 dark:border-slate-800 bg-slate-50 dark:bg-slate-900 text-sm text-slate-900 dark:text-slate-100 focus:outline-none focus:ring-2 focus:ring-emerald-500"
            />
          </div>

          {/* Filter pills */}
          <button
            onClick={onToggleUnreadOnly}
            className={`px-2.5 py-1.5 rounded-lg text-xs font-semibold border transition cursor-pointer ${
              unreadOnly
                ? 'bg-emerald-600 text-white border-emerald-600'
                : 'border-slate-200 dark:border-slate-800 text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-900'
            }`}
          >
            Unread
          </button>

          <button
            onClick={onToggleDedup}
            title="Collapse duplicate stories detected by Leo SimHash"
            className={`px-2.5 py-1.5 rounded-lg text-xs font-semibold border flex items-center gap-1 transition cursor-pointer ${
              dedup
                ? 'bg-violet-500/15 text-violet-700 dark:text-violet-300 border-violet-500/30'
                : 'border-slate-200 dark:border-slate-800 text-slate-500 hover:bg-slate-100 dark:hover:bg-slate-900'
            }`}
          >
            <Filter className="w-3 h-3" />
            Dedup
          </button>

          <select
            value={sort}
            onChange={(e) => onChangeSort(e.target.value as any)}
            className="px-2.5 py-1.5 rounded-lg text-xs font-semibold border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 text-slate-700 dark:text-slate-200 cursor-pointer"
          >
            <option value="newest">Newest</option>
            <option value="priority">Leo Priority</option>
            <option value="oldest">Oldest</option>
          </select>

          {/* View Switcher */}
          <div className="flex items-center rounded-lg border border-slate-200 dark:border-slate-800 p-0.5 bg-slate-50 dark:bg-slate-900">
            <button
              onClick={() => onChangeLayout('title')}
              title="Title-Only View"
              className={`p-1.5 rounded-md cursor-pointer ${
                layout === 'title' ? 'bg-white dark:bg-slate-800 text-emerald-600 shadow-xs' : 'text-slate-400'
              }`}
            >
              <AlignJustify className="w-3.5 h-3.5" />
            </button>
            <button
              onClick={() => onChangeLayout('magazine')}
              title="Magazine View"
              className={`p-1.5 rounded-md cursor-pointer ${
                layout === 'magazine' ? 'bg-white dark:bg-slate-800 text-emerald-600 shadow-xs' : 'text-slate-400'
              }`}
            >
              <LayoutList className="w-3.5 h-3.5" />
            </button>
            <button
              onClick={() => onChangeLayout('cards')}
              title="Cards View"
              className={`p-1.5 rounded-md cursor-pointer ${
                layout === 'cards' ? 'bg-white dark:bg-slate-800 text-emerald-600 shadow-xs' : 'text-slate-400'
              }`}
            >
              <LayoutGrid className="w-3.5 h-3.5" />
            </button>
          </div>

          <button
            onClick={onMarkAllRead}
            title="Mark all as read"
            className="p-2 rounded-lg border border-slate-200 dark:border-slate-800 text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-900 cursor-pointer"
          >
            <CheckCheck className="w-4 h-4" />
          </button>

          <button
            onClick={onRefresh}
            disabled={refreshing}
            title="Refresh feeds now"
            className="p-2 rounded-lg border border-slate-200 dark:border-slate-800 text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-900 cursor-pointer disabled:opacity-50"
          >
            <RefreshCw className={`w-4 h-4 ${refreshing ? 'animate-spin text-emerald-500' : ''}`} />
          </button>
        </div>
      </header>

      {/* Article Feed Content */}
      <div className="flex-1 overflow-y-auto px-6 py-4">
        {loading && articles.length === 0 ? (
          <div className="space-y-4 max-w-4xl mx-auto mt-6">
            {[1, 2, 3, 4, 5].map((i) => (
              <div key={i} className="h-24 rounded-2xl bg-slate-100 dark:bg-slate-900 animate-pulse" />
            ))}
          </div>
        ) : articles.length === 0 ? (
          <div className="text-center py-20 max-w-md mx-auto">
            <div className="w-12 h-12 rounded-2xl bg-slate-100 dark:bg-slate-900 text-slate-400 flex items-center justify-center mx-auto mb-3">
              <Sparkles className="w-6 h-6" />
            </div>
            <h3 className="text-base font-semibold text-slate-800 dark:text-slate-200">
              All caught up!
            </h3>
            <p className="text-sm text-slate-500 mt-1">
              No articles match the current view or filter. Try refreshing your feeds or switching filters.
            </p>
          </div>
        ) : layout === 'title' ? (
          /* TITLE-ONLY COMPACT VIEW */
          <div className="max-w-5xl mx-auto divide-y divide-slate-100 dark:divide-slate-900 border border-slate-200/80 dark:border-slate-800 rounded-2xl overflow-hidden">
            {articles.map((art) => {
              const active = art.id === selectedId;
              return (
                <div
                  key={art.id}
                  onClick={() => onSelectArticle(art.id)}
                  className={`px-4 py-2.5 flex items-center gap-3 cursor-pointer transition ${
                    active
                      ? 'bg-emerald-50 dark:bg-emerald-950/30'
                      : 'hover:bg-slate-50 dark:hover:bg-slate-900/70'
                  } ${art.is_read ? 'opacity-60' : ''}`}
                >
                  <span className="w-36 shrink-0 text-xs font-medium text-slate-500 truncate">
                    {art.feed_title}
                  </span>
                  {art.priority_score >= 65 && (
                    <span className="px-1.5 py-0.5 rounded bg-violet-500/15 text-violet-600 dark:text-violet-300 text-[10px] font-bold shrink-0">
                      {art.priority_score}
                    </span>
                  )}
                  <span className="flex-1 text-sm font-medium text-slate-900 dark:text-slate-100 truncate">
                    {art.title}
                  </span>
                  {art.duplicate_count > 1 && (
                    <span className="text-[11px] px-1.5 py-0.5 rounded-full bg-slate-100 dark:bg-slate-800 text-slate-500 shrink-0">
                      +{art.duplicate_count - 1} sources
                    </span>
                  )}
                  <span className="text-xs text-slate-400 shrink-0 w-16 text-right">
                    {formatRelativeTime(art.published_at)}
                  </span>
                  <div className="flex items-center gap-1 shrink-0">
                    <button
                      onClick={(e) => onToggleReadLater(art, e)}
                      className={`p-1 rounded hover:bg-slate-200 dark:hover:bg-slate-800 ${
                        art.is_read_later ? 'text-blue-500' : 'text-slate-400'
                      }`}
                    >
                      <Bookmark className="w-3.5 h-3.5" fill={art.is_read_later ? 'currentColor' : 'none'} />
                    </button>
                    <button
                      onClick={(e) => onToggleRead(art, e)}
                      className="p-1 rounded hover:bg-slate-200 dark:hover:bg-slate-800 text-slate-400"
                    >
                      <Check className="w-3.5 h-3.5" />
                    </button>
                  </div>
                </div>
              );
            })}
          </div>
        ) : layout === 'cards' ? (
          /* CARDS GRID VIEW */
          <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-5 max-w-6xl mx-auto">
            {articles.map((art) => {
              const active = art.id === selectedId;
              return (
                <article
                  key={art.id}
                  onClick={() => onSelectArticle(art.id)}
                  className={`group rounded-2xl border overflow-hidden flex flex-col cursor-pointer transition ${
                    active
                      ? 'border-emerald-500 ring-2 ring-emerald-500/20 bg-emerald-50/30 dark:bg-emerald-950/20'
                      : 'border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 hover:shadow-md'
                  } ${art.is_read ? 'opacity-65' : ''}`}
                >
                  {art.image_url && (
                    <div className="h-40 w-full overflow-hidden bg-slate-100 dark:bg-slate-800">
                      <img
                        src={art.image_url}
                        alt=""
                        loading="lazy"
                        className="w-full h-full object-cover group-hover:scale-105 transition duration-300"
                        onError={(e) => { (e.target as HTMLImageElement).parentElement!.style.display = 'none'; }}
                      />
                    </div>
                  )}
                  <div className="p-4 flex-1 flex flex-col">
                    <div className="flex items-center justify-between gap-2 text-xs text-slate-500 mb-2">
                      <span className="font-semibold truncate">{art.feed_title}</span>
                      <span className="shrink-0">{formatRelativeTime(art.published_at)}</span>
                    </div>
                    <h3 className="font-bold text-slate-900 dark:text-white leading-snug line-clamp-2 mb-2">
                      {art.title}
                    </h3>
                    <p className="text-xs text-slate-600 dark:text-slate-400 line-clamp-3 flex-1">
                      {art.summary}
                    </p>
                    <div className="mt-3 pt-3 border-t border-slate-100 dark:border-slate-800 flex items-center justify-between">
                      <div className="flex items-center gap-1.5">
                        {art.priority_score >= 65 && (
                          <span className="px-1.5 py-0.5 rounded bg-violet-500/15 text-violet-600 dark:text-violet-300 text-[10px] font-bold">
                            Leo {art.priority_score}
                          </span>
                        )}
                        <span className="text-[11px] text-slate-400">{art.reading_time_mins}m read</span>
                      </div>
                      <div className="flex items-center gap-1">
                        <button
                          onClick={(e) => onToggleReadLater(art, e)}
                          className={`p-1 rounded hover:bg-slate-100 dark:hover:bg-slate-800 ${
                            art.is_read_later ? 'text-blue-500' : 'text-slate-400'
                          }`}
                        >
                          <Bookmark className="w-3.5 h-3.5" fill={art.is_read_later ? 'currentColor' : 'none'} />
                        </button>
                        <button
                          onClick={(e) => onToggleRead(art, e)}
                          className="p-1 rounded hover:bg-slate-100 dark:hover:bg-slate-800 text-slate-400"
                        >
                          <Check className="w-3.5 h-3.5" />
                        </button>
                      </div>
                    </div>
                  </div>
                </article>
              );
            })}
          </div>
        ) : (
          /* MAGAZINE VIEW (FEEDLY CLASSIC DEFAULT) */
          <div className="max-w-4xl mx-auto space-y-3">
            {articles.map((art) => {
              const active = art.id === selectedId;
              return (
                <article
                  key={art.id}
                  onClick={() => onSelectArticle(art.id)}
                  className={`group p-4 rounded-2xl border flex gap-4 cursor-pointer transition ${
                    active
                      ? 'border-emerald-500 bg-emerald-50/40 dark:bg-emerald-950/20 ring-1 ring-emerald-500/30'
                      : 'border-slate-200/80 dark:border-slate-800 bg-white dark:bg-slate-900/50 hover:border-slate-300 dark:hover:border-slate-700'
                  } ${art.is_read ? 'opacity-65' : ''}`}
                >
                  {art.image_url && (
                    <div className="w-32 h-24 rounded-xl overflow-hidden bg-slate-100 dark:bg-slate-800 shrink-0 hidden sm:block">
                      <img
                        src={art.image_url}
                        alt=""
                        loading="lazy"
                        className="w-full h-full object-cover"
                        onError={(e) => { (e.target as HTMLImageElement).parentElement!.style.display = 'none'; }}
                      />
                    </div>
                  )}

                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2 flex-wrap mb-1">
                      {art.priority_score >= 65 && (
                        <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full bg-violet-500/15 text-violet-700 dark:text-violet-300 text-[11px] font-bold">
                          <Sparkles className="w-3 h-3" />
                          Leo {art.priority_score}
                        </span>
                      )}
                      <span className="text-xs font-semibold text-emerald-600 dark:text-emerald-400 truncate">
                        {art.feed_title}
                      </span>
                      <span className="text-xs text-slate-400">•</span>
                      <span className="text-xs text-slate-400">
                        {formatRelativeTime(art.published_at)}
                      </span>
                      <span className="text-xs text-slate-400">•</span>
                      <span className="text-xs text-slate-400">{art.reading_time_mins} min read</span>
                      {art.duplicate_count > 1 && (
                        <span className="text-[11px] px-2 py-0.5 rounded-full bg-amber-500/15 text-amber-700 dark:text-amber-300 font-medium">
                          +{art.duplicate_count - 1} related
                        </span>
                      )}
                    </div>

                    <h3 className="text-base font-bold text-slate-900 dark:text-white leading-snug group-hover:text-emerald-600 dark:group-hover:text-emerald-400 transition">
                      {art.title}
                    </h3>

                    <p className="text-sm text-slate-600 dark:text-slate-400 line-clamp-2 mt-1">
                      {art.summary}
                    </p>

                    <div className="mt-2.5 flex items-center justify-between gap-2">
                      <div className="flex items-center gap-1.5 flex-wrap">
                        {art.tags.map((tag) => (
                          <span
                            key={tag}
                            className="text-[11px] px-2 py-0.5 rounded-md bg-slate-100 dark:bg-slate-800 text-slate-600 dark:text-slate-300 font-medium"
                          >
                            #{tag}
                          </span>
                        ))}
                        {art.board_ids.length > 0 && (
                          <span className="inline-flex items-center gap-1 text-[11px] text-blue-500 font-medium">
                            <Layers className="w-3 h-3" /> {art.board_ids.length}
                          </span>
                        )}
                        {art.highlight_count > 0 && (
                          <span className="inline-flex items-center gap-1 text-[11px] text-amber-500 font-medium">
                            <Highlighter className="w-3 h-3" /> {art.highlight_count}
                          </span>
                        )}
                        {art.note_count > 0 && (
                          <span className="inline-flex items-center gap-1 text-[11px] text-emerald-500 font-medium">
                            <MessageSquare className="w-3 h-3" /> {art.note_count}
                          </span>
                        )}
                      </div>

                      <div className="flex items-center gap-1 opacity-70 group-hover:opacity-100 transition">
                        <button
                          onClick={(e) => onToggleReadLater(art, e)}
                          title="Read Later (l)"
                          className={`p-1.5 rounded-lg hover:bg-slate-100 dark:hover:bg-slate-800 cursor-pointer ${
                            art.is_read_later ? 'text-blue-500' : 'text-slate-400'
                          }`}
                        >
                          <Bookmark className="w-4 h-4" fill={art.is_read_later ? 'currentColor' : 'none'} />
                        </button>
                        <button
                          onClick={(e) => onToggleRead(art, e)}
                          title={art.is_read ? 'Mark Unread (m)' : 'Mark Read (m)'}
                          className="p-1.5 rounded-lg hover:bg-slate-100 dark:hover:bg-slate-800 text-slate-400 hover:text-emerald-600 cursor-pointer"
                        >
                          <Check className="w-4 h-4" />
                        </button>
                        <a
                          href={art.url}
                          target="_blank"
                          rel="noopener noreferrer"
                          onClick={(e) => e.stopPropagation()}
                          title="Open original (v)"
                          className="p-1.5 rounded-lg hover:bg-slate-100 dark:hover:bg-slate-800 text-slate-400 hover:text-slate-700 dark:hover:text-slate-200"
                        >
                          <ExternalLink className="w-4 h-4" />
                        </a>
                      </div>
                    </div>
                  </div>
                </article>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
};
