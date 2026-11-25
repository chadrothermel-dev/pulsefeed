export interface User {
  id: string;
  email: string;
  name: string;
  role: 'admin' | 'editor' | 'member' | 'viewer' | string;
  is_active?: boolean;
  created_at?: string;
}

export interface SidebarCounts {
  all_unread: number;
  today_unread: number;
  leo_priority: number;
  read_later: number;
  recently_read: number;
}

export interface Folder {
  id: string;
  name: string;
  color: string;
  icon: string;
  sort_order: number;
  unread_count: number;
  feed_count: number;
}

export interface Feed {
  id: string;
  folder_id: string | null;
  title: string;
  feed_url: string;
  site_url: string | null;
  description: string | null;
  icon_url: string | null;
  refresh_interval_mins: number;
  last_fetched_at: string | null;
  last_error: string | null;
  error_count: number;
  unread_count: number;
  total_articles: number;
  is_active: boolean;
  created_at: string;
}

export interface ArticleListItem {
  id: string;
  feed_id: string;
  feed_title: string;
  feed_icon: string | null;
  folder_id: string | null;
  title: string;
  url: string;
  author: string | null;
  summary: string;
  image_url: string | null;
  published_at: string;
  reading_time_mins: number;
  priority_score: number;
  is_muted: boolean;
  is_read: boolean;
  is_read_later: boolean;
  tags: string[];
  duplicate_count: number;
  duplicate_cluster_id: string | null;
  board_ids: string[];
  highlight_count: number;
  note_count: number;
  ai_summary: string | null;
  ai_sentiment: string | null;
}

export interface Highlight {
  id: string;
  article_id: string;
  user_id: string;
  user_name: string;
  selected_text: string;
  color: string;
  note: string | null;
  created_at: string;
}

export interface ArticleNote {
  id: string;
  article_id: string;
  user_id: string;
  user_name: string;
  content: string;
  created_at: string;
}

export interface DuplicateBrief {
  id: string;
  title: string;
  url: string;
  feed_title: string;
  published_at: string;
}

export interface ArticleDetail extends ArticleListItem {
  content_html: string;
  content_text: string;
  highlights: Highlight[];
  notes: ArticleNote[];
  duplicates: DuplicateBrief[];
}

export interface TeamBoard {
  id: string;
  name: string;
  description: string;
  visibility: 'private' | 'team' | 'public' | string;
  color: string;
  created_by: string;
  creator_name: string;
  article_count: number;
  public_token: string;
  created_at: string;
}

export interface LeoRule {
  id: string;
  name: string;
  rule_type: 'priority' | 'mute' | 'auto_tag' | 'auto_board' | 'alert' | string;
  keywords: string;
  match_field: 'any' | 'title' | 'content' | 'author' | string;
  feed_id: string | null;
  folder_id: string | null;
  score_delta: number;
  target_tag: string | null;
  target_board_id: string | null;
  target_webhook_id: string | null;
  is_enabled: boolean;
  match_count: number;
  created_at: string;
}

export interface WebhookEndpoint {
  id: string;
  name: string;
  kind: 'generic' | 'slack' | 'teams' | string;
  url: string;
  secret: string | null;
  is_enabled: boolean;
  last_status: string | null;
  last_triggered_at: string | null;
  created_at: string;
}

export interface ApiKeyRecord {
  id: string;
  user_id: string;
  user_email: string;
  name: string;
  key_prefix: string;
  scopes: string;
  last_used_at: string | null;
  created_at: string;
}

export interface AuditLogEntry {
  id: number;
  user_id: string | null;
  user_email: string | null;
  action: string;
  target_type: string;
  target_id: string | null;
  details: string;
  created_at: string;
}

export interface SystemSettings {
  llm_provider: string;
  llm_model: string;
  llm_api_key_set: boolean;
  llm_base_url: string;
  sso_enabled: boolean;
  sso_provider_name: string;
  sso_issuer_url: string;
  sso_client_id: string;
  dedup_threshold: number;
  default_poll_mins: number;
}

export interface TelemetryStats {
  total_feeds: number;
  active_feeds: number;
  failing_feeds: number;
  total_articles: number;
  unread_articles: number;
  total_users: number;
  total_boards: number;
  total_rules: number;
  total_webhooks: number;
  db_size_bytes: number;
}

export interface DiscoveredFeed {
  title: string;
  feed_url: string;
  site_url: string;
}

export type LayoutMode = 'magazine' | 'title' | 'cards';

export type ActiveNav =
  | { kind: 'view'; view: 'today' | 'all' | 'leo_priority' | 'read_later' | 'recently_read' | 'muted'; label: string }
  | { kind: 'folder'; folderId: string; label: string }
  | { kind: 'feed'; feedId: string; label: string }
  | { kind: 'board'; boardId: string; label: string }
  | { kind: 'leo_rules' }
  | { kind: 'admin' };
