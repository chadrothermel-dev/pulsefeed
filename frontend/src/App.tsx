import React, { useCallback, useEffect, useState } from 'react';
import { X, Keyboard } from 'lucide-react';
import { apiFetch, getToken, setToken } from './api';
import {
  ActiveNav,
  ArticleDetail,
  ArticleListItem,
  EditionInfo,
  Feed,
  Folder,
  LayoutMode,
  SidebarCounts,
  TeamBoard,
  User,
} from './types';
import { AuthScreen } from './components/AuthScreen';
import { Sidebar } from './components/Sidebar';
import { ArticleList } from './components/ArticleList';
import { ArticleReader } from './components/ArticleReader';
import { AddFeedModal } from './components/AddFeedModal';
import { LeoRulesPanel } from './components/LeoRulesPanel';
import { EnterpriseAdminPanel } from './components/EnterpriseAdminPanel';

export default function App() {
  const [edition, setEdition] = useState<'personal' | 'enterprise'>('personal');
  const [user, setUser] = useState<User | null>(null);
  const [authChecked, setAuthChecked] = useState(false);

  const [darkMode, setDarkMode] = useState<boolean>(() => {
    const saved = localStorage.getItem('pulsefeed_theme');
    return saved ? saved === 'dark' : true;
  });

  const [counts, setCounts] = useState<SidebarCounts>({
    all_unread: 0,
    today_unread: 0,
    leo_priority: 0,
    read_later: 0,
    recently_read: 0,
  });
  const [folders, setFolders] = useState<Folder[]>([]);
  const [feeds, setFeeds] = useState<Feed[]>([]);
  const [boards, setBoards] = useState<TeamBoard[]>([]);

  const [activeNav, setActiveNav] = useState<ActiveNav>({
    kind: 'view',
    view: 'today',
    label: 'Today',
  });

  const [articles, setArticles] = useState<ArticleListItem[]>([]);
  const [loadingArticles, setLoadingArticles] = useState(false);
  const [refreshing, setRefreshing] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');
  const [unreadOnly, setUnreadOnly] = useState(false);
  const [dedup, setDedup] = useState(true);
  const [sort, setSort] = useState<'newest' | 'priority' | 'oldest'>('newest');
  const [layout, setLayout] = useState<LayoutMode>('magazine');

  const [selectedArticleId, setSelectedArticleId] = useState<string | null>(null);
  const [readerArticle, setReaderArticle] = useState<ArticleDetail | null>(null);
  const [loadingReader, setLoadingReader] = useState(false);

  const [showAddFeed, setShowAddFeed] = useState(false);
  const [showNewFolder, setShowNewFolder] = useState(false);
  const [newFolderName, setNewFolderName] = useState('');
  const [showNewBoard, setShowNewBoard] = useState(false);
  const [newBoardName, setNewBoardName] = useState('');
  const [newBoardDesc, setNewBoardDesc] = useState('');
  const [newBoardVis, setNewBoardVis] = useState<'team' | 'public' | 'private'>('team');
  const [showShortcuts, setShowShortcuts] = useState(false);

  // Keep browser/app window title synced with active edition
  useEffect(() => {
    document.title =
      edition === 'personal'
        ? 'PulseFeed Personal — RSS & Intelligence Reader'
        : 'PulseFeed Enterprise — Intelligence & RSS Reader';
  }, [edition]);

  // Apply dark mode class
  useEffect(() => {
    const root = document.documentElement;
    if (darkMode) root.classList.add('dark');
    else root.classList.remove('dark');
    localStorage.setItem('pulsefeed_theme', darkMode ? 'dark' : 'light');
  }, [darkMode]);

  // Detect Personal vs Enterprise edition, handle OAuth callback token, and verify session
  useEffect(() => {
    const urlParams = new URLSearchParams(window.location.search);
    const oauthToken = urlParams.get('oauth_token');
    if (oauthToken) {
      setToken(oauthToken);
      window.history.replaceState({}, '', window.location.pathname);
    }

    apiFetch<EditionInfo>('/edition')
      .then((info) => {
        setEdition(info.edition);
        if (info.is_personal) {
          // Personal local mode: auto-authenticate seamlessly without login screen
          return apiFetch<User>('/auth/me').then((u) => setUser(u));
        }
        const token = getToken();
        if (!token) return;
        return apiFetch<User>('/auth/me')
          .then((u) => setUser(u))
          .catch(() => setToken(null));
      })
      .catch(() => {
        const token = getToken();
        if (token) {
          return apiFetch<User>('/auth/me')
            .then((u) => setUser(u))
            .catch(() => setToken(null));
        }
      })
      .finally(() => setAuthChecked(true));
  }, []);

  const loadSidebar = useCallback(async () => {
    const [sb, br] = await Promise.all([
      apiFetch<{ counts: SidebarCounts; folders: Folder[]; feeds: Feed[] }>('/sidebar'),
      apiFetch<{ boards: TeamBoard[] }>('/boards'),
    ]);
    setCounts(sb.counts);
    setFolders(sb.folders);
    setFeeds(sb.feeds);
    setBoards(br.boards);
  }, []);

  const loadArticles = useCallback(async () => {
    if (activeNav.kind === 'leo_rules' || activeNav.kind === 'admin') return;
    setLoadingArticles(true);
    try {
      const params = new URLSearchParams();
      params.set('limit', '60');
      params.set('dedup', String(dedup));
      params.set('sort', sort);
      if (unreadOnly) params.set('unread_only', 'true');
      if (searchQuery.trim()) params.set('q', searchQuery.trim());

      if (activeNav.kind === 'view') {
        params.set('view', activeNav.view);
      } else if (activeNav.kind === 'folder') {
        params.set('folder_id', activeNav.folderId);
      } else if (activeNav.kind === 'feed') {
        params.set('feed_id', activeNav.feedId);
      } else if (activeNav.kind === 'board') {
        params.set('board_id', activeNav.boardId);
      }

      const res = await apiFetch<{ articles: ArticleListItem[] }>(`/articles?${params.toString()}`);
      setArticles(res.articles);
    } finally {
      setLoadingArticles(false);
    }
  }, [activeNav, dedup, sort, unreadOnly, searchQuery]);

  useEffect(() => {
    if (!user) return;
    loadSidebar();
  }, [user, loadSidebar]);

  useEffect(() => {
    if (!user) return;
    const timer = setTimeout(() => {
      loadArticles();
    }, searchQuery ? 200 : 0);
    return () => clearTimeout(timer);
  }, [user, loadArticles, searchQuery]);

  const openArticle = useCallback(
    async (id: string) => {
      setSelectedArticleId(id);
      setLoadingReader(true);
      try {
        const detail = await apiFetch<ArticleDetail>(`/articles/${id}`);
        setReaderArticle(detail);
        if (!detail.is_read) {
          await apiFetch(`/articles/${id}/state`, {
            method: 'PATCH',
            body: JSON.stringify({ is_read: true }),
          });
          setArticles((prev) => prev.map((a) => (a.id === id ? { ...a, is_read: true } : a)));
          loadSidebar();
        }
      } finally {
        setLoadingReader(false);
      }
    },
    [loadSidebar]
  );

  const refreshReader = async (id: string) => {
    const detail = await apiFetch<ArticleDetail>(`/articles/${id}`);
    setReaderArticle(detail);
  };

  const handleToggleRead = async (art: ArticleListItem, e?: React.MouseEvent) => {
    e?.stopPropagation();
    const next = !art.is_read;
    await apiFetch(`/articles/${art.id}/state`, {
      method: 'PATCH',
      body: JSON.stringify({ is_read: next }),
    });
    setArticles((prev) => prev.map((a) => (a.id === art.id ? { ...a, is_read: next } : a)));
    loadSidebar();
  };

  const handleToggleReadLater = async (art: ArticleListItem, e?: React.MouseEvent) => {
    e?.stopPropagation();
    const next = !art.is_read_later;
    await apiFetch(`/articles/${art.id}/state`, {
      method: 'PATCH',
      body: JSON.stringify({ is_read_later: next }),
    });
    setArticles((prev) => prev.map((a) => (a.id === art.id ? { ...a, is_read_later: next } : a)));
    loadSidebar();
  };

  const handleRefreshAll = async () => {
    setRefreshing(true);
    try {
      if (activeNav.kind === 'feed') {
        await apiFetch(`/feeds/${activeNav.feedId}/refresh`, { method: 'POST' });
      } else {
        await apiFetch('/feeds/refresh-all', { method: 'POST' });
      }
      await Promise.all([loadSidebar(), loadArticles()]);
    } finally {
      setRefreshing(false);
    }
  };

  const handleMarkAllRead = async () => {
    const body: any = {};
    if (activeNav.kind === 'folder') body.folder_id = activeNav.folderId;
    if (activeNav.kind === 'feed') body.feed_id = activeNav.feedId;
    await apiFetch('/articles/mark-all-read', { method: 'POST', body: JSON.stringify(body) });
    await Promise.all([loadSidebar(), loadArticles()]);
  };

  const handleCreateFolder = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newFolderName.trim()) return;
    await apiFetch('/folders', {
      method: 'POST',
      body: JSON.stringify({ name: newFolderName.trim() }),
    });
    setNewFolderName('');
    setShowNewFolder(false);
    loadSidebar();
  };

  const handleCreateBoard = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newBoardName.trim()) return;
    await apiFetch('/boards', {
      method: 'POST',
      body: JSON.stringify({
        name: newBoardName.trim(),
        description: newBoardDesc.trim(),
        visibility: newBoardVis,
      }),
    });
    setNewBoardName('');
    setNewBoardDesc('');
    setShowNewBoard(false);
    loadSidebar();
  };

  const handleDeleteFeed = async (feed: Feed) => {
    if (!window.confirm(`Unfollow "${feed.title}" and remove its articles?`)) return;
    await apiFetch(`/feeds/${feed.id}`, { method: 'DELETE' });
    if (activeNav.kind === 'feed' && activeNav.feedId === feed.id) {
      setActiveNav({ kind: 'view', view: 'today', label: 'Today' });
    }
    setReaderArticle(null);
    await Promise.all([loadSidebar(), loadArticles()]);
  };

  const handleDeleteFolder = async (folder: Folder) => {
    if (!window.confirm(`Delete folder "${folder.name}"? Feeds inside will move to Uncategorized.`)) return;
    await apiFetch(`/folders/${folder.id}`, { method: 'DELETE' });
    if (activeNav.kind === 'folder' && activeNav.folderId === folder.id) {
      setActiveNav({ kind: 'view', view: 'today', label: 'Today' });
    }
    await Promise.all([loadSidebar(), loadArticles()]);
  };

  const handleDeleteBoard = async (board: TeamBoard) => {
    if (!window.confirm(`Delete board "${board.name}"?`)) return;
    await apiFetch(`/boards/${board.id}`, { method: 'DELETE' });
    if (activeNav.kind === 'board' && activeNav.boardId === board.id) {
      setActiveNav({ kind: 'view', view: 'today', label: 'Today' });
    }
    await Promise.all([loadSidebar(), loadArticles()]);
  };

  // Feedly-style Keyboard Shortcuts (j, k, m, l, v, /, ?, Esc)
  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      const target = e.target as HTMLElement | null;
      const inInput = target && ['INPUT', 'TEXTAREA', 'SELECT'].includes(target.tagName);
      if (e.key === 'Escape') {
        setReaderArticle(null);
        setSelectedArticleId(null);
        setShowAddFeed(false);
        setShowShortcuts(false);
        return;
      }
      if (inInput) return;

      if (e.key === '/') {
        e.preventDefault();
        document.getElementById('pulsefeed-search-input')?.focus();
      } else if (e.key === '?') {
        setShowShortcuts((s) => !s);
      } else if (e.key === 'j' && articles.length > 0) {
        const idx = articles.findIndex((a) => a.id === selectedArticleId);
        const next = articles[Math.min(idx + 1, articles.length - 1)];
        if (next) openArticle(next.id);
      } else if (e.key === 'k' && articles.length > 0) {
        const idx = articles.findIndex((a) => a.id === selectedArticleId);
        const prev = articles[Math.max(idx - 1, 0)];
        if (prev) openArticle(prev.id);
      } else if (e.key === 'm' && selectedArticleId) {
        const art = articles.find((a) => a.id === selectedArticleId);
        if (art) handleToggleRead(art);
      } else if (e.key === 'l' && selectedArticleId) {
        const art = articles.find((a) => a.id === selectedArticleId);
        if (art) handleToggleReadLater(art);
      } else if (e.key === 'v' && selectedArticleId) {
        const art = articles.find((a) => a.id === selectedArticleId);
        if (art) window.open(art.url, '_blank', 'noopener,noreferrer');
      }
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [articles, selectedArticleId, openArticle]);

  if (!authChecked) {
    return <div className="min-h-screen bg-slate-950" />;
  }

  if (!user) {
    return <AuthScreen onSuccess={(u) => setUser(u)} />;
  }

  const currentBoard =
    activeNav.kind === 'board' ? boards.find((b) => b.id === activeNav.boardId) : undefined;
  const currentFeed =
    activeNav.kind === 'feed' ? feeds.find((f) => f.id === activeNav.feedId) : undefined;
  const currentFolder =
    activeNav.kind === 'folder' ? folders.find((f) => f.id === activeNav.folderId) : undefined;

  const deleteCurrentLabel = currentFeed
    ? 'Unfollow Feed'
    : currentBoard
    ? 'Delete Board'
    : currentFolder
    ? 'Delete Folder'
    : undefined;

  const handleDeleteCurrent = currentFeed
    ? () => handleDeleteFeed(currentFeed)
    : currentBoard
    ? () => handleDeleteBoard(currentBoard)
    : currentFolder
    ? () => handleDeleteFolder(currentFolder)
    : undefined;

  const headerTitle =
    activeNav.kind === 'view' || activeNav.kind === 'folder' || activeNav.kind === 'feed' || activeNav.kind === 'board'
      ? activeNav.label
      : 'PulseFeed';

  const headerSubtitle =
    activeNav.kind === 'board' && currentBoard
      ? `${currentBoard.description || 'Team Board'} • Visibility: ${currentBoard.visibility}`
      : activeNav.kind === 'view' && activeNav.view === 'leo_priority'
      ? 'High-signal articles scored 65+ by your Leo AI rules'
      : undefined;

  return (
    <div className="flex h-screen overflow-hidden bg-white dark:bg-slate-950 text-slate-900 dark:text-slate-100">
      <Sidebar
        edition={edition}
        user={user}
        counts={counts}
        folders={folders}
        feeds={feeds}
        boards={boards}
        activeNav={activeNav}
        onSelectNav={(nav) => {
          setActiveNav(nav);
          setReaderArticle(null);
        }}
        onOpenAddFeed={() => setShowAddFeed(true)}
        onOpenNewFolder={() => setShowNewFolder(true)}
        onOpenNewBoard={() => setShowNewBoard(true)}
        onDeleteFeed={handleDeleteFeed}
        onDeleteFolder={handleDeleteFolder}
        onDeleteBoard={handleDeleteBoard}
        onOpenShortcuts={() => setShowShortcuts(true)}
        darkMode={darkMode}
        onToggleDarkMode={() => setDarkMode(!darkMode)}
        onLogout={() => {
          setToken(null);
          setUser(null);
        }}
      />

      {activeNav.kind === 'leo_rules' ? (
        <LeoRulesPanel
          edition={edition}
          boards={boards}
          onChanged={() => {
            loadSidebar();
            loadArticles();
          }}
        />
      ) : activeNav.kind === 'admin' ? (
        <EnterpriseAdminPanel edition={edition} currentUser={user} />
      ) : (
        <ArticleList
          title={headerTitle}
          subtitle={headerSubtitle}
          board={currentBoard}
          articles={articles}
          selectedId={selectedArticleId}
          loading={loadingArticles}
          refreshing={refreshing}
          searchQuery={searchQuery}
          onSearchChange={setSearchQuery}
          unreadOnly={unreadOnly}
          onToggleUnreadOnly={() => setUnreadOnly(!unreadOnly)}
          dedup={dedup}
          onToggleDedup={() => setDedup(!dedup)}
          sort={sort}
          onChangeSort={setSort}
          layout={layout}
          onChangeLayout={setLayout}
          onRefresh={handleRefreshAll}
          onMarkAllRead={handleMarkAllRead}
          deleteLabel={deleteCurrentLabel}
          onDeleteCurrent={handleDeleteCurrent}
          onSelectArticle={openArticle}
          onToggleRead={handleToggleRead}
          onToggleReadLater={handleToggleReadLater}
        />
      )}

      {(readerArticle || loadingReader) && (
        <ArticleReader
          edition={edition}
          article={readerArticle}
          loading={loadingReader}
          boards={boards}
          onClose={() => {
            setReaderArticle(null);
            setSelectedArticleId(null);
          }}
          onRefreshArticle={refreshReader}
          onStateChanged={() => {
            loadSidebar();
            loadArticles();
          }}
        />
      )}

      {showAddFeed && (
        <AddFeedModal
          folders={folders}
          onClose={() => setShowAddFeed(false)}
          onChanged={() => {
            loadSidebar();
            loadArticles();
          }}
        />
      )}

      {/* New Folder Modal */}
      {showNewFolder && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/50 p-4">
          <form
            onSubmit={handleCreateFolder}
            className="w-full max-w-sm rounded-2xl bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 p-6 space-y-4 shadow-2xl"
          >
            <div className="flex items-center justify-between">
              <h3 className="font-bold">Create Feed Folder</h3>
              <button type="button" onClick={() => setShowNewFolder(false)} className="cursor-pointer">
                <X className="w-4 h-4 text-slate-400" />
              </button>
            </div>
            <input
              type="text"
              required
              autoFocus
              value={newFolderName}
              onChange={(e) => setNewFolderName(e.target.value)}
              placeholder="e.g. Threat Intelligence"
              className="w-full px-3.5 py-2.5 rounded-xl border border-slate-200 dark:border-slate-700 bg-slate-50 dark:bg-slate-800 text-sm"
            />
            <button
              type="submit"
              className="w-full py-2.5 rounded-xl bg-emerald-600 hover:bg-emerald-700 text-white text-sm font-semibold cursor-pointer"
            >
              Create Folder
            </button>
          </form>
        </div>
      )}

      {/* New Team Board Modal */}
      {showNewBoard && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/50 p-4">
          <form
            onSubmit={handleCreateBoard}
            className="w-full max-w-md rounded-2xl bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 p-6 space-y-4 shadow-2xl"
          >
            <div className="flex items-center justify-between">
              <h3 className="font-bold">
                {edition === 'personal' ? 'Create Saved Board' : 'Create Enterprise Team Board'}
              </h3>
              <button type="button" onClick={() => setShowNewBoard(false)} className="cursor-pointer">
                <X className="w-4 h-4 text-slate-400" />
              </button>
            </div>
            <input
              type="text"
              required
              autoFocus
              value={newBoardName}
              onChange={(e) => setNewBoardName(e.target.value)}
              placeholder="Board Name (e.g. Competitor Watch)"
              className="w-full px-3.5 py-2.5 rounded-xl border border-slate-200 dark:border-slate-700 bg-slate-50 dark:bg-slate-800 text-sm"
            />
            <input
              type="text"
              value={newBoardDesc}
              onChange={(e) => setNewBoardDesc(e.target.value)}
              placeholder="Short description..."
              className="w-full px-3.5 py-2.5 rounded-xl border border-slate-200 dark:border-slate-700 bg-slate-50 dark:bg-slate-800 text-sm"
            />
            <select
              value={newBoardVis}
              onChange={(e) => setNewBoardVis(e.target.value as any)}
              className="w-full px-3.5 py-2.5 rounded-xl border border-slate-200 dark:border-slate-700 bg-slate-50 dark:bg-slate-800 text-sm"
            >
              <option value="team">Team Visible (Shared with workspace)</option>
              <option value="public">Public (Generates shareable RSS feed URL)</option>
              <option value="private">Private (Only you)</option>
            </select>
            <button
              type="submit"
              className="w-full py-2.5 rounded-xl bg-emerald-600 hover:bg-emerald-700 text-white text-sm font-semibold cursor-pointer"
            >
              Create Board
            </button>
          </form>
        </div>
      )}

      {/* Keyboard Shortcuts Modal */}
      {showShortcuts && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/50 p-4">
          <div className="w-full max-w-sm rounded-2xl bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 p-6 shadow-2xl">
            <div className="flex items-center justify-between mb-4">
              <h3 className="font-bold flex items-center gap-2">
                <Keyboard className="w-4 h-4 text-emerald-500" />
                Keyboard Shortcuts
              </h3>
              <button onClick={() => setShowShortcuts(false)} className="cursor-pointer">
                <X className="w-4 h-4 text-slate-400" />
              </button>
            </div>
            <div className="space-y-2 text-sm">
              {[
                ['j / k', 'Next / Previous article'],
                ['m', 'Toggle Read / Unread'],
                ['l', 'Toggle Read Later'],
                ['v', 'Open original article in new tab'],
                ['/', 'Focus full-text search'],
                ['Esc', 'Close reader or modal'],
                ['?', 'Toggle this shortcuts panel'],
              ].map(([key, desc]) => (
                <div key={key} className="flex items-center justify-between">
                  <span className="text-slate-600 dark:text-slate-400">{desc}</span>
                  <kbd className="px-2 py-0.5 rounded bg-slate-100 dark:bg-slate-800 font-mono text-xs font-bold">
                    {key}
                  </kbd>
                </div>
              ))}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
