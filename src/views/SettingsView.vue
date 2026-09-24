<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { ElMessage } from 'element-plus'
import { Folder } from '@element-plus/icons-vue'
import { api } from '../api'

const apiKey = ref('')
const downloadDir = ref('')
const saving = ref(false)

onMounted(async () => {
  try {
    const s = await api.getSettings()
    apiKey.value = s.apiKey
    downloadDir.value = s.downloadDir
  } catch (e) {
    ElMessage.error(String(e))
  }
})

async function browse() {
  try {
    const dir = await api.pickDownloadDir()
    if (dir) downloadDir.value = dir
  } catch (e) {
    ElMessage.error(String(e))
  }
}

async function save() {
  if (!downloadDir.value.trim()) {
    ElMessage.warning('请选择视频保存目录')
    return
  }
  saving.value = true
  try {
    await api.saveSettings(apiKey.value.trim(), downloadDir.value)
    ElMessage.success('设置已保存')
  } catch (e) {
    ElMessage.error(String(e))
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <div class="settings">
    <header class="head">
      <h1>设置</h1>
      <p>配置生成服务与本地存储</p>
    </header>

    <div class="panel">
      <div class="panel-head">
        <div class="panel-icon"><el-icon :size="17"><Key /></el-icon></div>
        <div class="panel-titles">
          <span class="panel-title">MiniMax API</span>
          <span class="panel-desc">视频生成能力来自 MiniMax 开放平台</span>
        </div>
      </div>
      <div class="field">
        <label>API Key</label>
        <el-input
          v-model="apiKey"
          type="password"
          placeholder="在 MiniMax 开放平台「账户管理 → API Keys」中创建"
          show-password
        />
        <div class="field-tip">
          <el-icon :size="13"><Lock /></el-icon>
          <span>Key 仅保存在本机应用数据目录中，不会上传到任何服务器</span>
        </div>
      </div>
    </div>

    <div class="panel">
      <div class="panel-head">
        <div class="panel-icon icon-blue"><el-icon :size="17"><FolderOpened /></el-icon></div>
        <div class="panel-titles">
          <span class="panel-title">下载设置</span>
          <span class="panel-desc">生成的视频文件保存位置</span>
        </div>
      </div>
      <div class="field">
        <label>保存目录</label>
        <div class="dir-row">
          <el-input v-model="downloadDir" placeholder="选择一个文件夹用于保存视频" readonly />
          <el-button class="ghost-btn" :icon="Folder" @click="browse">浏览</el-button>
        </div>
      </div>
    </div>

    <div class="foot">
      <el-button type="primary" size="large" round :loading="saving" @click="save">
        保存设置
      </el-button>
    </div>
  </div>
</template>

<style scoped>
.settings {
  max-width: 680px;
  margin: 0 auto;
}

.head h1 {
  margin: 0;
  font-size: 24px;
  font-weight: 700;
  letter-spacing: 0.3px;
}

.head p {
  margin: 6px 0 22px;
  font-size: 13px;
  color: var(--text-secondary);
}

.panel {
  background: var(--bg-card);
  border: 1px solid var(--border-base);
  border-radius: var(--radius-lg);
  padding: 20px 22px;
  margin-bottom: 16px;
}

.panel-head {
  display: flex;
  align-items: center;
  gap: 13px;
  padding-bottom: 16px;
  margin-bottom: 16px;
  border-bottom: 1px solid var(--border-base);
}

.panel-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 38px;
  height: 38px;
  border-radius: 11px;
  background: var(--brand-soft);
  color: var(--brand-1);
  flex-shrink: 0;
}

.panel-icon.icon-blue {
  background: rgba(77, 162, 255, 0.14);
  color: #4da2ff;
}

.panel-titles {
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.panel-title {
  font-size: 14.5px;
  font-weight: 600;
  color: var(--text-primary);
}

.panel-desc {
  font-size: 12px;
  color: var(--text-muted);
}

.field > label {
  display: block;
  font-size: 12.5px;
  color: var(--text-secondary);
  margin-bottom: 8px;
}

.field-tip {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-top: 9px;
  font-size: 11.5px;
  color: var(--text-muted);
}

.dir-row {
  display: flex;
  gap: 10px;
}

.dir-row .el-input {
  flex: 1;
}

.ghost-btn {
  background: var(--bg-card-2);
  border: 1px solid var(--border-strong);
  color: var(--text-regular);
}

.ghost-btn:hover {
  border-color: rgba(124, 92, 255, 0.5);
  background: var(--bg-hover);
  color: var(--text-primary);
}

.foot {
  text-align: center;
  padding-top: 8px;
}
</style>
