<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Promotion } from '@element-plus/icons-vue'
import { api } from '../api'
import { store } from '../store'

const router = useRouter()

const prompt = ref('')
const model = ref('MiniMax-Hailuo-2.3')
const resolution = ref('768P')
const duration = ref(6)
const optimizer = ref(true)
const submitting = ref(false)

const examples = [
  '一只橘猫在洒满阳光的窗台上打盹，毛发细节清晰，暖色调，电影质感',
  '[拉远] 无人机视角下的雪山日出，云海翻涌，光线穿透云层，史诗感',
  '赛博朋克风格的雨夜街道，霓虹灯牌倒映在积水中，行人撑伞走过',
]

const activeTasks = computed(() =>
  store.videos.filter((v) => v.status === 'submitted' || v.status === 'processing'),
)

function setResolution(r: string) {
  resolution.value = r
  if (r === '1080P') duration.value = 6
}

function applyExample(text: string) {
  prompt.value = text
}

async function submit() {
  if (!prompt.value.trim()) {
    ElMessage.warning('请输入视频描述')
    return
  }

  try {
    const settings = await api.getSettings()
    if (!settings.apiKey.trim()) {
      ElMessageBox.confirm(
        '尚未配置 MiniMax API Key，无法提交生成任务。是否现在前往设置页配置？',
        '需要先配置 API Key',
        {
          confirmButtonText: '前往设置',
          cancelButtonText: '稍后再说',
          type: 'warning',
        },
      )
        .then(() => router.push('/settings'))
        .catch(() => {})
      return
    }
  } catch {
    void 0
  }

  submitting.value = true
  try {
    const video = await api.createVideo({
      prompt: prompt.value,
      model: model.value,
      duration: duration.value,
      resolution: resolution.value,
      promptOptimizer: optimizer.value,
    })
    store.replace(video)
    ElMessage.success('任务已提交，生成完成后可在「视频库」播放')
    prompt.value = ''
  } catch (e) {
    ElMessage({
      message: String(e),
      type: 'error',
      duration: 0,
      showClose: true,
      grouping: true,
    })
  } finally {
    submitting.value = false
  }
}
</script>

<template>
  <div class="studio">
    <section class="creator">
      <header class="head">
        <h1>创作视频</h1>
        <p>描述你的画面，Hailuo 2.3 为你生成电影级短片</p>
      </header>

      <div class="panel">
        <div class="panel-head">
          <span class="panel-title">视频描述</span>
          <span class="panel-count">{{ prompt.length }} / 2000</span>
        </div>
        <el-input
          v-model="prompt"
          type="textarea"
          :rows="7"
          maxlength="2000"
          resize="none"
          class="prompt-input"
          placeholder="描述你想生成的视频内容，支持运镜指令语法，例如：[推近] 一只橘猫在洒满阳光的窗台上打盹，毛发细节清晰，暖色调，电影质感"
        />
        <div class="examples">
          <span class="examples-label">灵感示例</span>
          <div class="example-chips">
            <button
              v-for="ex in examples"
              :key="ex"
              type="button"
              class="chip-example"
              @click="applyExample(ex)"
            >
              {{ ex }}
            </button>
          </div>
        </div>
      </div>

      <div class="panel">
        <div class="panel-head">
          <span class="panel-title">生成参数</span>
        </div>
        <div class="field">
          <label>分辨率</label>
          <div class="choice-row">
            <button
              type="button"
              class="choice"
              :class="{ active: resolution === '768P' }"
              @click="setResolution('768P')"
            >
              <span class="choice-title">768P</span>
              <span class="choice-desc">支持 6 / 10 秒</span>
            </button>
            <button
              type="button"
              class="choice"
              :class="{ active: resolution === '1080P' }"
              @click="setResolution('1080P')"
            >
              <span class="choice-title">1080P</span>
              <span class="choice-desc">高清 · 仅 6 秒</span>
            </button>
          </div>
        </div>
        <div class="field">
          <label>时长</label>
          <div class="chip-row">
            <button
              type="button"
              class="chip"
              :class="{ active: duration === 6 }"
              @click="duration = 6"
            >
              6 秒
            </button>
            <button
              type="button"
              class="chip"
              :class="{ active: duration === 10 }"
              :disabled="resolution === '1080P'"
              @click="duration = 10"
            >
              10 秒
            </button>
          </div>
        </div>
        <div class="field-inline">
          <div class="field-inline-text">
            <span class="optimizer-name">智能优化提示词</span>
            <span class="optimizer-desc">由模型自动润色描述，通常出片效果更好</span>
          </div>
          <el-switch v-model="optimizer" />
        </div>
      </div>

      <button type="button" class="submit-btn" :disabled="submitting" @click="submit">
        <el-icon v-if="submitting" class="is-loading"><Loading /></el-icon>
        <el-icon v-else><Promotion /></el-icon>
        {{ submitting ? '提交中…' : '开始生成' }}
      </button>
    </section>

    <aside class="queue">
      <header class="queue-head">
        <h2>生成队列</h2>
        <span v-if="activeTasks.length" class="queue-count">
          {{ activeTasks.length }} 个进行中
        </span>
      </header>

      <div v-if="activeTasks.length" class="queue-list">
        <article v-for="task in activeTasks" :key="task.id" class="task-card">
          <div class="task-glow" />
          <div class="task-top">
            <span class="task-badge">
              <el-icon v-if="task.status === 'processing'" class="is-loading">
                <Loading />
              </el-icon>
              {{ task.status === 'processing' ? '生成中' : '排队中' }}
            </span>
            <span class="task-meta">
              {{ task.resolution }} · {{ task.duration }} 秒 · #{{ task.id }}
            </span>
          </div>
          <p class="task-prompt">{{ task.prompt }}</p>
          <div class="task-bar">
            <div class="task-bar-fill" />
          </div>
          <div class="task-foot">每 10 秒自动刷新 · 通常 1–3 分钟完成</div>
        </article>
      </div>

      <div v-else class="queue-empty">
        <div class="queue-empty-icon">
          <el-icon :size="26"><MagicStick /></el-icon>
        </div>
        <p class="queue-empty-title">暂无进行中的任务</p>
        <span class="queue-empty-desc">提交左侧描述后，这里会实时展示生成进度</span>
      </div>
    </aside>
  </div>
</template>

<style scoped>
.studio {
  display: grid;
  grid-template-columns: 480px minmax(0, 1fr);
  gap: 26px;
  align-items: start;
  max-width: 1280px;
  margin: 0 auto;
}

.head h1 {
  margin: 0;
  font-size: 24px;
  font-weight: 700;
  letter-spacing: 0.3px;
}

.head p {
  margin: 6px 0 0;
  font-size: 13px;
  color: var(--text-secondary);
}

.creator {
  display: flex;
  flex-direction: column;
  gap: 18px;
}

.panel {
  background: var(--bg-card);
  border: 1px solid var(--border-base);
  border-radius: var(--radius-lg);
  padding: 18px 20px 20px;
  transition: border-color 0.2s ease;
}

.panel:focus-within {
  border-color: rgba(124, 92, 255, 0.45);
}

.panel-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}

.panel-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.panel-count {
  font-size: 12px;
  color: var(--text-muted);
  font-variant-numeric: tabular-nums;
}

.prompt-input :deep(.el-textarea__inner) {
  font-size: 14px;
  line-height: 1.7;
  padding: 12px 14px;
}

.examples {
  margin-top: 14px;
}

.examples-label {
  font-size: 12px;
  color: var(--text-muted);
}

.example-chips {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-top: 8px;
}

.chip-example {
  text-align: left;
  padding: 8px 12px;
  font-size: 12.5px;
  line-height: 1.5;
  color: var(--text-secondary);
  background: var(--bg-input);
  border: 1px solid var(--border-base);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition:
    color 0.18s ease,
    border-color 0.18s ease;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  font-family: inherit;
}

.chip-example:hover {
  color: var(--text-primary);
  border-color: rgba(124, 92, 255, 0.5);
}

.field {
  margin-bottom: 16px;
}

.field > label {
  display: block;
  font-size: 12.5px;
  color: var(--text-secondary);
  margin-bottom: 9px;
}

.choice-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
}

.choice {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 4px;
  padding: 12px 14px;
  background: var(--bg-input);
  border: 1px solid var(--border-base);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition:
    border-color 0.18s ease,
    background 0.18s ease;
  font-family: inherit;
}

.choice:hover {
  border-color: var(--border-strong);
}

.choice.active {
  border-color: rgba(124, 92, 255, 0.75);
  background: var(--brand-soft);
}

.choice-title {
  font-size: 14px;
  font-weight: 700;
  color: var(--text-primary);
}

.choice.active .choice-title {
  color: #c9baff;
}

.choice-desc {
  font-size: 11.5px;
  color: var(--text-muted);
}

.chip-row {
  display: flex;
  gap: 8px;
}

.chip {
  min-width: 72px;
  height: 34px;
  border-radius: 17px;
  font-size: 13px;
  font-family: inherit;
  color: var(--text-secondary);
  background: var(--bg-input);
  border: 1px solid var(--border-base);
  cursor: pointer;
  transition:
    all 0.18s ease;
}

.chip:hover:not(:disabled) {
  color: var(--text-primary);
  border-color: var(--border-strong);
}

.chip.active {
  color: #fff;
  background: var(--brand-gradient);
  border-color: transparent;
  font-weight: 600;
}

.chip:disabled {
  opacity: 0.35;
  cursor: not-allowed;
}

.field-inline {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 12px 14px;
  background: var(--bg-input);
  border: 1px solid var(--border-base);
  border-radius: var(--radius-md);
}

.field-inline-text {
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.optimizer-name {
  font-size: 13.5px;
  color: var(--text-primary);
  font-weight: 500;
}

.optimizer-desc {
  font-size: 11.5px;
  color: var(--text-muted);
}

.submit-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 9px;
  height: 50px;
  border: none;
  border-radius: var(--radius-md);
  background: var(--brand-gradient);
  color: #fff;
  font-size: 15.5px;
  font-weight: 700;
  letter-spacing: 1px;
  cursor: pointer;
  font-family: inherit;
  box-shadow: 0 10px 28px rgba(124, 92, 255, 0.35);
  transition:
    transform 0.15s ease,
    box-shadow 0.2s ease,
    filter 0.2s ease;
}

.submit-btn:hover:not(:disabled) {
  filter: brightness(1.08);
  transform: translateY(-1px);
  box-shadow: 0 14px 34px rgba(124, 92, 255, 0.45);
}

.submit-btn:active:not(:disabled) {
  transform: translateY(0);
}

.submit-btn:disabled {
  opacity: 0.65;
  cursor: not-allowed;
}

.queue {
  display: flex;
  flex-direction: column;
  min-height: 420px;
}

.queue-head {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 16px;
}

.queue-head h2 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
}

.queue-count {
  font-size: 12px;
  color: #c9baff;
  background: var(--brand-soft);
  padding: 3px 10px;
  border-radius: 10px;
  font-weight: 600;
}

.queue-list {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.task-card {
  position: relative;
  overflow: hidden;
  background: var(--bg-card);
  border: 1px solid var(--border-base);
  border-radius: var(--radius-lg);
  padding: 16px 18px;
}

.task-glow {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 2px;
  background: var(--brand-gradient);
  opacity: 0.85;
}

.task-top {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.task-badge {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  font-weight: 600;
  color: #c9baff;
  background: var(--brand-soft);
  padding: 4px 10px;
  border-radius: 20px;
  flex-shrink: 0;
}

.task-meta {
  font-size: 12px;
  color: var(--text-muted);
  white-space: nowrap;
}

.task-prompt {
  margin: 12px 0 0;
  font-size: 13.5px;
  line-height: 1.65;
  color: var(--text-regular);
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.task-bar {
  position: relative;
  height: 4px;
  margin-top: 14px;
  border-radius: 2px;
  background: rgba(255, 255, 255, 0.06);
  overflow: hidden;
}

.task-bar-fill {
  position: absolute;
  top: 0;
  bottom: 0;
  width: 40%;
  border-radius: 2px;
  background: var(--brand-gradient);
  animation: slide 1.6s ease-in-out infinite;
}

@keyframes slide {
  0% {
    left: -40%;
  }
  100% {
    left: 100%;
  }
}

.task-foot {
  margin-top: 10px;
  font-size: 11.5px;
  color: var(--text-muted);
}

.queue-empty {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  border: 1px dashed var(--border-strong);
  border-radius: var(--radius-lg);
  padding: 60px 24px;
  text-align: center;
}

.queue-empty-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 58px;
  height: 58px;
  border-radius: 18px;
  background: var(--brand-soft);
  color: var(--brand-1);
  margin-bottom: 6px;
}

.queue-empty-title {
  margin: 0;
  font-size: 14.5px;
  font-weight: 600;
  color: var(--text-regular);
}

.queue-empty-desc {
  font-size: 12.5px;
  color: var(--text-muted);
}

@media (max-width: 1100px) {
  .studio {
    grid-template-columns: 1fr;
  }
}
</style>
