export type VideoStatus = 'submitted' | 'processing' | 'success' | 'fail'

export interface Video {
  id: number
  prompt: string
  model: string
  duration: number
  resolution: string
  promptOptimizer: boolean
  status: VideoStatus
  taskId: string | null
  fileId: string | null
  failReason: string | null
  localPath: string | null
  fileSize: number | null
  createdAt: string
  updatedAt: string
}

export interface Settings {
  apiKey: string
  downloadDir: string
}

export const STATUS_MAP: Record<
  VideoStatus,
  { label: string; tag: 'info' | 'warning' | 'success' | 'danger' }
> = {
  submitted: { label: '排队中', tag: 'info' },
  processing: { label: '生成中', tag: 'warning' },
  success: { label: '已完成', tag: 'success' },
  fail: { label: '失败', tag: 'danger' },
}

export function formatSize(bytes: number | null): string {
  if (!bytes) return '-'
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`
}
