<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'
import { convertFileSrc } from '@tauri-apps/api/core'
import { ElMessage, ElMessageBox } from 'element-plus'
import { api } from '../api'
import { store } from '../store'
import { STATUS_MAP, formatSize, type Video } from '../types'

const router = useRouter()
const keyword = ref('')
const filtered = computed(() => {
  const kw = keyword.value.trim().toLowerCase()
  if (!kw) return store.videos
  return store.videos.filter((v) => v.prompt.toLowerCase().includes(kw))
})
const doneCount = computed(() => store.videos.filter((v) => v.localPath).length)

const playing = ref<Video | null>(null)
const editing = ref<Video | null>(null)
const editPrompt = ref('')
const downloadingId = ref<number | null>(null)
const retryingId = ref<number | null>(null)

function src(v: Video) {
  return v.localPath ? convertFileSrc(v.localPath) : ''
}

function folder(v: Video) {
  return v.localPath ? v.localPath.replace(/[\\/][^\\/]+$/, '') : ''
}

async function download(v: Video) {
  downloadingId.value = v.id
  try {
    store.replace(await api.downloadVideo(v.id))
    ElMessage.success('视频已下载到本地')
  } catch (e) {
    ElMessage.error(String(e))
  } finally {
    downloadingId.value = null
  }
}

async function retry(v: Video) {
  retryingId.value = v.id
  try {
    store.replace(await api.retryVideo(v.id))
    ElMessage.success('已重新提交生成任务')
  } catch (e) {
    ElMessage.error(String(e))
  } finally {
    retryingId.value = null
  }
}

function openEdit(v: Video) {
  editing.value = v
  editPrompt.value = v.prompt
}

async function saveEdit() {
  if (!editing.value) return
  try {
    store.replace(await api.updateVideo(editing.value.id, editPrompt.value))
    editing.value = null
    ElMessage.success('描述已更新')
  } catch (e) {
    ElMessage.error(String(e))
  }
}

async function removeRecord(v: Video) {
  try {
    await ElMessageBox.confirm(
      '仅删除该条生成记录，本地视频文件将保留。',
      '删除记录',
      { type: 'warning', confirmButtonText: '删除记录', cancelButtonText: '取消' },
    )
  } catch {
    return
  }
  try {
    await api.deleteVideo(v.id, false)
    store.remove(v.id)
    ElMessage.success('记录已删除')
  } catch (e) {
    ElMessage.error(String(e))
  }
}

async function removeWithFile(v: Video) {
  if (!v.localPath) return
  try {
    await ElMessageBox.confirm(
      `将同时删除本地视频文件：${v.localPath}`,
      '删除记录和文件',
      { type: 'warning', confirmButtonText: '全部删除', cancelButtonText: '取消' },
    )
  } catch {
    return
  }
  try {
    await api.deleteVideo(v.id, true)
    store.remove(v.id)
    ElMessage.success('记录和文件已删除')
  } catch (e) {
    ElMessage.error(String(e))
  }
}

async function openFolder(v: Video) {
  if (!v.localPath) return
  try {
    await api.openPath(folder(v))
  } catch (e) {
    ElMessage.error(String(e))
  }
}

function onCommand(cmd: string, v: Video) {
  switch (cmd) {
    case 'download':
      download(v)
      break
    case 'edit':
      openEdit(v)
      break
    case 'folder':
      openFolder(v)
      break
    case 'delRecord':
      removeRecord(v)
      break
    case 'delFile':
      removeWithFile(v)
      break
  }
}

async function refresh() {
  try {
    await store.refresh()
    ElMessage.success('已刷新')
  } catch (e) {
    ElMessage.error(String(e))
  }
}
</script>

<template>
  <div class="library">
    <header class="head">
      <div class="head-text">
        <h1>视频库</h1>
        <p>{{ filtered.length }} 个作品 · {{ doneCount }} 个已保存到本地</p>
      </div>
      <div class="head-actions">
        <el-input
          v-model="keyword"
          placeholder="搜索视频描述…"
          clearable
          class="search"
          :prefix-icon="'Search'"
        />
        <el-button class="ghost-btn" :icon="'Refresh'" @click="refresh">刷新</el-button>
      </div>
    </header>

    <div v-if="filtered.length" class="grid">
      <article v-for="v in filtered" :key="v.id" class="card">
        <div class="thumb" @click="v.localPath && (playing = v)">
          <video
            v-if="v.localPath"
            :src="src(v)"
            preload="metadata"
            muted
            class="thumb-video"
          />
          <div v-else class="ph">
            <el-icon :size="30"><Film /></el-icon>
            <span>{{ STATUS_MAP[v.status].label }}</span>
          </div>
          <div v-if="v.localPath" class="thumb-mask">
            <div class="play-btn">
              <el-icon :size="20"><VideoPlay /></el-icon>
            </div>
          </div>
          <span class="pill pill-status" :data-status="v.status">
            {{ STATUS_MAP[v.status].label }}
          </span>
          <span v-if="v.localPath" class="pill pill-res">{{ v.resolution }}</span>
        </div>

        <div class="info">
          <p class="prompt" :title="v.prompt">{{ v.prompt }}</p>
          <div class="meta">
            {{ v.model.replace('MiniMax-', '') }} · {{ v.duration }} 秒 ·
            {{ formatSize(v.fileSize) }}
          </div>
          <div class="meta">{{ v.createdAt }}</div>
          <div v-if="v.status === 'fail' && v.failReason" class="fail" :title="v.failReason">
            {{ v.failReason }}
          </div>
          <div class="foot">
            <el-button
              v-if="v.localPath"
              size="small"
              type="primary"
              round
              @click="playing = v"
            >
              播放
            </el-button>
            <el-button
              v-else-if="v.status === 'success'"
              size="small"
              type="primary"
              round
              :loading="downloadingId === v.id"
              @click="download(v)"
            >
              下载
            </el-button>
            <el-button
              v-if="v.status === 'fail'"
              size="small"
              round
              class="retry-btn"
              :loading="retryingId === v.id"
              @click="retry(v)"
            >
              重试
            </el-button>
            <el-dropdown
              trigger="click"
              class="more"
              @command="(cmd: string) => onCommand(cmd, v)"
            >
              <button type="button" class="more-btn">
                <el-icon :size="16"><MoreFilled /></el-icon>
              </button>
              <template #dropdown>
                <el-dropdown-menu>
                  <el-dropdown-item command="download" :disabled="v.status !== 'success'">
                    下载视频
                  </el-dropdown-item>
                  <el-dropdown-item command="edit">编辑描述</el-dropdown-item>
                  <el-dropdown-item command="folder" :disabled="!v.localPath">
                    打开文件夹
                  </el-dropdown-item>
                  <el-dropdown-item command="delRecord" divided>删除记录</el-dropdown-item>
                  <el-dropdown-item command="delFile" :disabled="!v.localPath">
                    删除记录和文件
                  </el-dropdown-item>
                </el-dropdown-menu>
              </template>
            </el-dropdown>
          </div>
        </div>
      </article>
    </div>

    <div v-else class="empty">
      <div class="empty-icon">
        <el-icon :size="30"><Film /></el-icon>
      </div>
      <p class="empty-title">
        {{ keyword ? '没有找到匹配的视频' : '还没有作品' }}
      </p>
      <span class="empty-desc">
        {{ keyword ? '换个关键词试试' : '去创作页生成你的第一个视频吧' }}
      </span>
      <el-button v-if="!keyword" type="primary" round @click="router.push('/generate')">
        去创作
      </el-button>
    </div>

    <el-dialog v-model="playing" :title="`播放 #${playing?.id}`" width="760px" destroy-on-close>
      <video
        v-if="playing?.localPath"
        :src="src(playing)"
        controls
        autoplay
        class="player"
      />
      <div v-if="playing" class="play-prompt">{{ playing.prompt }}</div>
    </el-dialog>

    <el-dialog v-model="editing" title="编辑视频描述" width="560px">
      <el-input
        v-model="editPrompt"
        type="textarea"
        :rows="5"
        maxlength="2000"
        show-word-limit
      />
      <template #footer>
        <el-button @click="editing = null">取消</el-button>
        <el-button type="primary" @click="saveEdit">保存</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<style scoped>
.library {
  max-width: 1280px;
  margin: 0 auto;
}

.head {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: 20px;
  flex-wrap: wrap;
  margin-bottom: 22px;
}

.head-text h1 {
  margin: 0;
  font-size: 24px;
  font-weight: 700;
  letter-spacing: 0.3px;
}

.head-text p {
  margin: 6px 0 0;
  font-size: 13px;
  color: var(--text-secondary);
}

.head-actions {
  display: flex;
  gap: 10px;
}

.search {
  width: 300px;
}

.ghost-btn {
  background: var(--bg-card);
  border: 1px solid var(--border-base);
  color: var(--text-regular);
}

.ghost-btn:hover {
  border-color: var(--border-strong);
  background: var(--bg-hover);
  color: var(--text-primary);
}

.grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 20px;
}

.card {
  background: var(--bg-card);
  border: 1px solid var(--border-base);
  border-radius: var(--radius-lg);
  overflow: hidden;
  transition:
    transform 0.22s ease,
    border-color 0.22s ease,
    box-shadow 0.22s ease;
}

.card:hover {
  transform: translateY(-4px);
  border-color: rgba(124, 92, 255, 0.45);
  box-shadow: var(--shadow-card);
}

.thumb {
  position: relative;
  aspect-ratio: 16 / 9;
  background: #08090c;
  cursor: pointer;
  overflow: hidden;
}

.thumb-video {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
  transition: transform 0.35s ease;
}

.card:hover .thumb-video {
  transform: scale(1.04);
}

.ph {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 10px;
  color: var(--text-muted);
  font-size: 13px;
}

.thumb-mask {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(180deg, rgba(8, 9, 12, 0.05), rgba(8, 9, 12, 0.55));
  opacity: 0;
  transition: opacity 0.22s ease;
}

.card:hover .thumb-mask {
  opacity: 1;
}

.play-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 54px;
  height: 54px;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.14);
  backdrop-filter: blur(8px);
  border: 1px solid rgba(255, 255, 255, 0.3);
  color: #fff;
  transform: scale(0.85);
  transition: transform 0.22s ease;
}

.card:hover .play-btn {
  transform: scale(1);
}

.pill {
  position: absolute;
  top: 10px;
  font-size: 11px;
  font-weight: 600;
  padding: 3px 9px;
  border-radius: 12px;
  backdrop-filter: blur(6px);
}

.pill-status {
  left: 10px;
  color: #fff;
  background: rgba(20, 24, 34, 0.72);
  border: 1px solid rgba(255, 255, 255, 0.14);
}

.pill-status[data-status='processing'] {
  color: #ffc069;
}

.pill-status[data-status='fail'] {
  color: #ff8f8f;
}

.pill-status[data-status='success'] {
  color: #7ce7ae;
}

.pill-res {
  right: 10px;
  color: rgba(255, 255, 255, 0.85);
  background: rgba(20, 24, 34, 0.72);
  border: 1px solid rgba(255, 255, 255, 0.14);
}

.info {
  padding: 14px 16px 16px;
}

.prompt {
  margin: 0;
  font-size: 13.5px;
  line-height: 1.55;
  color: var(--text-primary);
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  min-height: 42px;
}

.meta {
  margin-top: 6px;
  font-size: 12px;
  color: var(--text-secondary);
}

.fail {
  margin-top: 6px;
  font-size: 12px;
  color: #ff8f8f;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.foot {
  margin-top: 13px;
  display: flex;
  align-items: center;
  gap: 8px;
}

.retry-btn {
  background: var(--bg-card-2);
  border: 1px solid var(--border-strong);
  color: #ffc069;
}

.retry-btn:hover {
  border-color: rgba(255, 192, 105, 0.6);
  background: var(--bg-hover);
  color: #ffd191;
}

.more {
  margin-left: auto;
}

.more-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
  border-radius: 50%;
  border: 1px solid var(--border-base);
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.18s ease;
}

.more-btn:hover {
  color: var(--text-primary);
  border-color: var(--border-strong);
  background: var(--bg-hover);
}

.empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 90px 24px;
  border: 1px dashed var(--border-strong);
  border-radius: var(--radius-lg);
  text-align: center;
}

.empty-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 62px;
  height: 62px;
  border-radius: 20px;
  background: var(--brand-soft);
  color: var(--brand-1);
  margin-bottom: 6px;
}

.empty-title {
  margin: 0;
  font-size: 15px;
  font-weight: 600;
  color: var(--text-regular);
}

.empty-desc {
  font-size: 12.5px;
  color: var(--text-muted);
  margin-bottom: 12px;
}

.player {
  width: 100%;
  border-radius: var(--radius-md);
  background: #000;
  display: block;
}

.play-prompt {
  margin-top: 12px;
  font-size: 13px;
  color: var(--text-secondary);
  line-height: 1.65;
}
</style>
