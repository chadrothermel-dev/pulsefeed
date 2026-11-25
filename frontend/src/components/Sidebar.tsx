import React, { useState } from 'react';
import {
  Rss,
  Sun,
  Inbox,
  Sparkles,
  Bookmark,
  History,
  VolumeX,
  Folder as FolderIcon,
  ChevronRight,
  ChevronDown,
  Plus,
  Layers,
  Shield,
  Sliders,
  AlertTriangle,
  Globe,
  Users,
  Lock,
  LogOut,
  Moon,
  HelpCircle,
} from 'lucide-react';
import { ActiveNav, Feed, Folder, SidebarCounts, TeamBoard, User } from '../types';

interface Props {
  user: User;
  counts: SidebarCounts;
  folders: Folder[];
  feeds: Feed[];
  boards: TeamBoard[];
  activeNav: ActiveNav;
  onSelectNav: (nav: ActiveNav) => void;
  onOpenAddFeed: () => void;
  onOpenNewFolder: () => void;
  onOpenNewBoard: () => void;
  onOpenShortcuts: () => void;
  darkMode: boolean;
  onToggleDarkMode: () => void;
  onLogout: () => void;
}

export const Sidebar: React.FC<Props> = ({
  user,
  counts,
  folders,
  feeds,
  boards,
  activeNav,
  onSelectNav,
  onOpenAddFeed,
  onOpenNewFolder,
  onOpenNewBoard,
  onOpenShortcuts,
  darkMode,
  onToggleDarkMode,
  onLogout,
}) => {
  const [expandedFolders, setExpandedFolders] = useState<Record<string, boolean>>({});

  const toggleFolder = (id: string, e: React.MouseEvent) => {
    e.stopPropagation();
    setExpandedFolders((prev) => ({ ...prev, [id]: !prev[id] }));
  };

  const isNavActive = (check: ActiveNav) => {
    if (check.kind !== activeNav.kind) return false;
    if (check.kind === 'view' && activeNav.kind === 'view') return check.view === activeNav.view;
    if (check.kind === 'folder' && activeNav.kind === 'folder') return check.folderId === activeNav.folderId;
    if (check.kind === 'feed' && activeNav.kind === 'feed') return check.feedId === activeNav.feedId;
    if (check.kind === 'board' && activeNav.kind === 'board') return check.boardId === activeNav.boardId;
    return true;
  };

  const navBtnClass = (active: boolean) =>
    `w-full flex items-center justify-between px-3 py-2 rounded-xl text-sm transition cursor-pointer ${
      active
        ? 'bg-emerald-500/15 text-emerald-700 dark:text-emerald-300 font-semibold'
        : 'text-slate-600 dark:text-slate-300 hover:bg-slate-200/60 dark:hover:bg-slate-800/70 font-medium'
    }`;

  const uncategorizedFeeds = feeds.filter((f) => !f.folder_id);

  return (
    <aside className="w-72 shrink-0 h-screen flex flex-col bg-slate-100/90 dark:bg-slate-900 border-r border-slate-200 dark:border-slate-800 select-none">
      {/* Brand Header */}
      <div className="px-4 py-4 flex items-center justify-between border-b border-slate-200/80 dark:border-slate-800">
        <div className="flex items-center gap-2.5">
          <div className="w-8 h-8 rounded-xl bg-emerald-600 text-white flex items-center justify-center shadow-sm">
            <Rss className="w-4 h-4" />
          </div>
          <div>
            <div className="font-bold text-slate-900 dark:text-white leading-tight text-base">
              PulseFeed
            </div>
            <div className="text-[10px] font-semibold uppercase tracking-wider text-emerald-600 dark:text-emerald-400">
              Enterprise
            </div>
          </div>
        </div>
        <button
          onClick={onOpenAddFeed}
          title="Add Feed or Import OPML"
          className="px-2.5 py-1.5 rounded-lg bg-emerald-600 hover:bg-emerald-700 text-white text-xs font-semibold flex items-center gap-1 shadow-sm transition cursor-pointer"
        >
          <Plus className="w-3.5 h-3.5" />
          Follow
        </button>
      </div>

      {/* Scrollable Nav */}
      <div className="flex-1 overflow-y-auto px-2.5 py-3 space-y-5">
        {/* Core Views */}
        <div className="space-y-0.5">
          <button
            onClick={() => onSelectNav({ kind: 'view', view: 'today', label: 'Today' })}
            className={navBtnClass(isNavActive({ kind: 'view', view: 'today', label: 'Today' }))}
          >
            <span className="flex items-center gap-2.5">
              <Sun className="w-4 h-4 text-amber-500" />
              Today
            </span>
            {counts.today_unread > 0 && (
              <span className="text-xs px-1.5 py-0.5 rounded-md bg-slate-200/80 dark:bg-slate-800 text-slate-700 dark:text-slate-300 font-semibold">
                {counts.today_unread}
              </span>
            )}
          </button>

          <button
            onClick={() => onSelectNav({ kind: 'view', view: 'leo_priority', label: 'Leo Priority' })}
            className={navBtnClass(isNavActive({ kind: 'view', view: 'leo_priority', label: 'Leo Priority' }))}
          >
            <span className="flex items-center gap-2.5">
              <Sparkles className="w-4 h-4 text-violet-500" />
              Leo Priority
            </span>
            {counts.leo_priority > 0 && (
              <span className="text-xs px-1.5 py-0.5 rounded-md bg-violet-500/15 text-violet-600 dark:text-violet-300 font-semibold">
                {counts.leo_priority}
              </span>
            )}
          </button>

          <button
            onClick={() => onSelectNav({ kind: 'view', view: 'all', label: 'All Articles' })}
            className={navBtnClass(isNavActive({ kind: 'view', view: 'all', label: 'All Articles' }))}
          >
            <span className="flex items-center gap-2.5">
              <Inbox className="w-4 h-4 text-emerald-500" />
              All Articles
            </span>
            {counts.all_unread > 0 && (
              <span className="text-xs px-1.5 py-0.5 rounded-md bg-slate-200/80 dark:bg-slate-800 text-slate-700 dark:text-slate-300 font-semibold">
                {counts.all_unread}
              </span>
            )}
          </button>

          <button
            onClick={() => onSelectNav({ kind: 'view', view: 'read_later', label: 'Read Later' })}
            className={navBtnClass(isNavActive({ kind: 'view', view: 'read_later', label: 'Read Later' }))}
          >
            <span className="flex items-center gap-2.5">
              <Bookmark className="w-4 h-4 text-blue-500" />
              Read Later
            </span>
            {counts.read_later > 0 && (
              <span className="text-xs text-slate-500 font-medium">{counts.read_later}</span>
            )}
          </button>

          <button
            onClick={() => onSelectNav({ kind: 'view', view: 'recently_read', label: 'Recently Read' })}
            className={navBtnClass(isNavActive({ kind: 'view', view: 'recently_read', label: 'Recently Read' }))}
          >
            <span className="flex items-center gap-2.5">
              <History className="w-4 h-4 text-slate-400" />
              Recently Read
            </span>
          </button>

          <button
            onClick={() => onSelectNav({ kind: 'view', view: 'muted', label: 'Muted by Leo' })}
            className={navBtnClass(isNavActive({ kind: 'view', view: 'muted', label: 'Muted by Leo' }))}
          >
            <span className="flex items-center gap-2.5">
              <VolumeX className="w-4 h-4 text-slate-400" />
              Muted by Leo
            </span>
          </button>
        </div>

        {/* Team Boards */}
        <div>
          <div className="px-3 mb-1.5 flex items-center justify-between">
            <span className="text-[11px] font-bold uppercase tracking-wider text-slate-400">
              Team Boards
            </span>
            <button
              onClick={onOpenNewBoard}
              title="New Team Board"
              className="p-0.5 rounded hover:bg-slate-200 dark:hover:bg-slate-800 text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 cursor-pointer"
            >
              <Plus className="w-3.5 h-3.5" />
            </button>
          </div>
          <div className="space-y-0.5">
            {boards.map((board) => (
              <button
                key={board.id}
                onClick={() => onSelectNav({ kind: 'board', boardId: board.id, label: board.name })}
                className={navBtnClass(isNavActive({ kind: 'board', boardId: board.id, label: board.name }))}
              >
                <span className="flex items-center gap-2.5 truncate">
                  <Layers className="w-4 h-4 shrink-0" style={{ color: board.color }} />
                  <span className="truncate">{board.name}</span>
                  {board.visibility === 'public' ? (
                    <Globe className="w-3 h-3 text-emerald-500 shrink-0" />
                  ) : board.visibility === 'team' ? (
                    <Users className="w-3 h-3 text-blue-400 shrink-0" />
                  ) : (
                    <Lock className="w-3 h-3 text-slate-400 shrink-0" />
                  )}
                </span>
                <span className="text-xs text-slate-400 font-medium">{board.article_count}</span>
              </button>
            ))}
          </div>
        </div>

        {/* Folders & Unlimited Feeds */}
        <div>
          <div className="px-3 mb-1.5 flex items-center justify-between">
            <span className="text-[11px] font-bold uppercase tracking-wider text-slate-400">
              Feeds ({feeds.length})
            </span>
            <button
              onClick={onOpenNewFolder}
              title="New Folder"
              className="p-0.5 rounded hover:bg-slate-200 dark:hover:bg-slate-800 text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 cursor-pointer"
            >
              <Plus className="w-3.5 h-3.5" />
            </button>
          </div>

          <div className="space-y-0.5">
            {folders.map((folder) => {
              const folderFeeds = feeds.filter((f) => f.folder_id === folder.id);
              const isOpen = expandedFolders[folder.id] ?? true;
              return (
                <div key={folder.id}>
                  <button
                    onClick={() => onSelectNav({ kind: 'folder', folderId: folder.id, label: folder.name })}
                    className={navBtnClass(
                      isNavActive({ kind: 'folder', folderId: folder.id, label: folder.name })
                    )}
                  >
                    <span className="flex items-center gap-1.5 truncate">
                      <span
                        onClick={(e) => toggleFolder(folder.id, e)}
                        className="p-0.5 rounded hover:bg-slate-300/50 dark:hover:bg-slate-700"
                      >
                        {isOpen ? <ChevronDown className="w-3.5 h-3.5" /> : <ChevronRight className="w-3.5 h-3.5" />}
                      </span>
                      <FolderIcon className="w-4 h-4 shrink-0" style={{ color: folder.color }} />
                      <span className="truncate">{folder.name}</span>
                    </span>
                    {folder.unread_count > 0 && (
                      <span className="text-xs text-slate-500 font-semibold">{folder.unread_count}</span>
                    )}
                  </button>

                  {isOpen && folderFeeds.length > 0 && (
                    <div className="ml-5 pl-2 border-l border-slate-200 dark:border-slate-800 space-y-0.5 mt-0.5">
                      {folderFeeds.map((feed) => (
                        <button
                          key={feed.id}
                          onClick={() => onSelectNav({ kind: 'feed', feedId: feed.id, label: feed.title })}
                          className={`w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg text-xs transition cursor-pointer ${
                            isNavActive({ kind: 'feed', feedId: feed.id, label: feed.title })
                              ? 'bg-emerald-500/15 text-emerald-700 dark:text-emerald-300 font-semibold'
                              : 'text-slate-600 dark:text-slate-400 hover:bg-slate-200/50 dark:hover:bg-slate-800/60'
                          }`}
                        >
                          <span className="flex items-center gap-2 truncate">
                            {feed.icon_url ? (
                              <img
                                src={feed.icon_url}
                                alt=""
                                className="w-3.5 h-3.5 rounded-sm shrink-0 object-cover"
                                onError={(e) => { (e.target as HTMLImageElement).style.display = 'none'; }}
                              />
                            ) : (
                              <Rss className="w-3 h-3 text-slate-400 shrink-0" />
                            )}
                            <span className="truncate">{feed.title}</span>
                            {feed.error_count > 0 && (
                              <AlertTriangle className="w-3 h-3 text-amber-500 shrink-0" />
                            )}
                          </span>
                          {feed.unread_count > 0 && (
                            <span className="text-[11px] font-semibold text-slate-400 ml-1">
                              {feed.unread_count}
                            </span>
                          )}
                        </button>
                      ))}
                    </div>
                  )}
                </div>
              );
            })}

            {uncategorizedFeeds.map((feed) => (
              <button
                key={feed.id}
                onClick={() => onSelectNav({ kind: 'feed', feedId: feed.id, label: feed.title })}
                className={navBtnClass(isNavActive({ kind: 'feed', feedId: feed.id, label: feed.title }))}
              >
                <span className="flex items-center gap-2 truncate">
                  <Rss className="w-3.5 h-3.5 text-slate-400 shrink-0" />
                  <span className="truncate">{feed.title}</span>
                </span>
                {feed.unread_count > 0 && (
                  <span className="text-xs text-slate-400 font-semibold">{feed.unread_count}</span>
                )}
              </button>
            ))}
          </div>
        </div>

        {/* Enterprise Tools */}
        <div>
          <div className="px-3 mb-1.5">
            <span className="text-[11px] font-bold uppercase tracking-wider text-slate-400">
              Intelligence &amp; Enterprise
            </span>
          </div>
          <div className="space-y-0.5">
            <button
              onClick={() => onSelectNav({ kind: 'leo_rules' })}
              className={navBtnClass(isNavActive({ kind: 'leo_rules' }))}
            >
              <span className="flex items-center gap-2.5">
                <Sliders className="w-4 h-4 text-violet-500" />
                Leo AI Rules
              </span>
            </button>
            <button
              onClick={() => onSelectNav({ kind: 'admin' })}
              className={navBtnClass(isNavActive({ kind: 'admin' }))}
            >
              <span className="flex items-center gap-2.5">
                <Shield className="w-4 h-4 text-emerald-500" />
                Enterprise Console
              </span>
            </button>
          </div>
        </div>
      </div>

      {/* Footer User Bar */}
      <div className="p-3 border-t border-slate-200 dark:border-slate-800 flex items-center justify-between gap-2">
        <div className="flex items-center gap-2.5 min-w-0">
          <div className="w-8 h-8 rounded-full bg-emerald-600/15 text-emerald-700 dark:text-emerald-300 font-bold text-xs flex items-center justify-center shrink-0 uppercase">
            {user.name.slice(0, 2)}
          </div>
          <div className="min-w-0">
            <div className="text-xs font-semibold text-slate-800 dark:text-slate-100 truncate">
              {user.name}
            </div>
            <div className="text-[10px] uppercase tracking-wider font-semibold text-slate-400">
              {user.role}
            </div>
          </div>
        </div>
        <div className="flex items-center gap-1">
          <button
            onClick={onOpenShortcuts}
            title="Keyboard Shortcuts (?)"
            className="p-1.5 rounded-lg text-slate-500 hover:bg-slate-200 dark:hover:bg-slate-800 cursor-pointer"
          >
            <HelpCircle className="w-4 h-4" />
          </button>
          <button
            onClick={onToggleDarkMode}
            title="Toggle Theme"
            className="p-1.5 rounded-lg text-slate-500 hover:bg-slate-200 dark:hover:bg-slate-800 cursor-pointer"
          >
            {darkMode ? <Sun className="w-4 h-4" /> : <Moon className="w-4 h-4" />}
          </button>
          <button
            onClick={onLogout}
            title="Sign Out"
            className="p-1.5 rounded-lg text-slate-500 hover:text-red-500 hover:bg-slate-200 dark:hover:bg-slate-800 cursor-pointer"
          >
            <LogOut className="w-4 h-4" />
          </button>
        </div>
      </div>
    </aside>
  );
};
