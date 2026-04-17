export interface MediaItem {
  id: string;
  title: string;
  year: number | null;
  duration_secs: number | null;
  size_bytes: number;
  container: string | null;
  resolution: string | null;
  file_url: string;
  file_url_https: string;
  play_url: string;
  playlist_url: string;
  dir: string;
  filename: string;
  series_key: string | null;
}

export interface RecentItem {
  media_id: string;
  title: string;
  played_at: number;
  file_url: string;
  file_url_https: string;
  play_url: string;
}
