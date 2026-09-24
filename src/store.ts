import { reactive } from 'vue'
import { ElNotification } from 'element-plus'
import { api } from './api'
import type { Video } from './types'

export const store = reactive({
  videos: [] as Video[],
  async refresh() {
    this.videos = await api.listVideos()
  },
  replace(video: Video) {
    const idx = this.videos.findIndex((v) => v.id === video.id)
    if (idx >= 0) this.videos[idx] = video
    else this.videos.unshift(video)
  },
  remove(id: number) {
    this.videos = this.videos.filter((v) => v.id !== id)
  },
})

let timer: number | null = null

export function startPolling() {
  if (timer !== null) return
  timer = window.setInterval(async () => {
    const active = store.videos.filter(
      (v) => v.status === 'submitted' || v.status === 'processing',
    )
    for (const task of active) {
      try {
        const prev = task.status
        const updated = await api.pollVideo(task.id)
        store.replace(updated)
        if (prev !== 'success' && updated.status === 'success') {
          ElNotification.success({
            title: '视频生成完成',
            message: task.prompt.slice(0, 60),
          })
        }
        if (prev !== 'fail' && updated.status === 'fail') {
          ElNotification.error({
            title: '视频生成失败',
            message: updated.failReason ?? '未知原因',
          })
        }
      } catch (e) {
        console.error('轮询任务状态失败', e)
      }
    }
  }, 10_000)
}
