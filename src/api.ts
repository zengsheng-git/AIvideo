import { invoke } from '@tauri-apps/api/core'
import type { Settings, Video } from './types'

export interface CreateVideoParams {
  [key: string]: unknown
  prompt: string
  model: string
  duration: number
  resolution: string
  promptOptimizer: boolean
}

export const api = {
  getSettings: () => invoke<Settings>('get_settings'),
  saveSettings: (apiKey: string, downloadDir: string) =>
    invoke<void>('save_settings', { apiKey, downloadDir }),
  pickDownloadDir: () => invoke<string | null>('pick_download_dir'),
  createVideo: (params: CreateVideoParams) => invoke<Video>('create_video', params),
  pollVideo: (id: number) => invoke<Video>('poll_video', { id }),
  retryVideo: (id: number) => invoke<Video>('retry_video', { id }),
  downloadVideo: (id: number) => invoke<Video>('download_video', { id }),
  listVideos: () => invoke<Video[]>('list_videos'),
  updateVideo: (id: number, prompt: string) => invoke<Video>('update_video', { id, prompt }),
  deleteVideo: (id: number, deleteFile: boolean) =>
    invoke<void>('delete_video', { id, deleteFile }),
  openPath: (path: string) => invoke<void>('open_path', { path }),
}
