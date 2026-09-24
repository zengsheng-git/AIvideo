<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { store, startPolling } from './store'

const activeCount = computed(
  () =>
    store.videos.filter((v) => v.status === 'submitted' || v.status === 'processing').length,
)

const navs = [
  { path: '/generate', label: '创作视频', icon: 'VideoCamera' },
  { path: '/library', label: '视频库', icon: 'Film' },
  { path: '/settings', label: '设置', icon: 'Setting' },
]

onMounted(async () => {
  try {
    await store.refresh()
  } catch (e) {
    console.error('加载视频列表失败', e)
  }
  startPolling()
})
</script>

<template>
  <div class="layout">
    <div class="glow glow-a" />
    <div class="glow glow-b" />
    <aside class="sidebar">
      <div class="brand">
        <div class="brand-mark">
          <el-icon :size="19"><VideoCamera /></el-icon>
        </div>
        <div class="brand-text">
          <span class="brand-name">AI 视频工坊</span>
          <span class="brand-sub">Hailuo 2.3 · MiniMax</span>
        </div>
      </div>
      <nav class="nav">
        <router-link
          v-for="n in navs"
          :key="n.path"
          :to="n.path"
          class="nav-item"
          active-class="active"
        >
          <el-icon :size="17"><component :is="n.icon" /></el-icon>
          <span class="nav-label">{{ n.label }}</span>
          <span v-if="n.path === '/library' && activeCount" class="nav-badge">
            {{ activeCount }}
          </span>
        </router-link>
      </nav>
      <div class="side-foot">
        <div class="side-dot" />
        <span>本地创作 · 数据不出设备</span>
      </div>
    </aside>
    <main class="main">
      <router-view />
    </main>
  </div>
</template>

<style scoped>
.layout {
  position: relative;
  display: flex;
  height: 100vh;
  overflow: hidden;
  background: var(--bg-page);
}

.glow {
  position: fixed;
  border-radius: 50%;
  filter: blur(120px);
  pointer-events: none;
  z-index: 0;
}

.glow-a {
  top: -180px;
  right: -120px;
  width: 520px;
  height: 520px;
  background: rgba(124, 92, 255, 0.13);
}

.glow-b {
  bottom: -200px;
  left: 160px;
  width: 460px;
  height: 460px;
  background: rgba(77, 162, 255, 0.09);
}

.sidebar {
  position: relative;
  z-index: 1;
  display: flex;
  flex-direction: column;
  width: 232px;
  flex-shrink: 0;
  background: var(--bg-sidebar);
  border-right: 1px solid var(--border-base);
  padding: 20px 14px 16px;
}

.brand {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 4px 8px 22px;
}

.brand-mark {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  border-radius: 12px;
  background: var(--brand-gradient);
  color: #fff;
  box-shadow: 0 6px 18px rgba(124, 92, 255, 0.35);
  flex-shrink: 0;
}

.brand-text {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.brand-name {
  font-size: 16px;
  font-weight: 700;
  color: var(--text-primary);
  letter-spacing: 0.5px;
  white-space: nowrap;
}

.brand-sub {
  font-size: 11px;
  color: var(--text-muted);
  white-space: nowrap;
}

.nav {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 11px;
  height: 44px;
  padding: 0 13px;
  border-radius: 10px;
  font-size: 14px;
  color: var(--text-secondary);
  text-decoration: none;
  transition:
    background 0.18s ease,
    color 0.18s ease;
}

.nav-item:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.nav-item.active {
  background: var(--brand-soft);
  color: #c9baff;
  font-weight: 600;
}

.nav-item.active .el-icon {
  color: var(--brand-1);
}

.nav-label {
  flex: 1;
}

.nav-badge {
  min-width: 20px;
  height: 20px;
  padding: 0 6px;
  border-radius: 10px;
  background: var(--brand-gradient);
  color: #fff;
  font-size: 11px;
  font-weight: 700;
  display: flex;
  align-items: center;
  justify-content: center;
}

.side-foot {
  margin-top: auto;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 10px;
  border-top: 1px solid var(--border-base);
  font-size: 11.5px;
  color: var(--text-muted);
}

.side-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: #35d07f;
  box-shadow: 0 0 8px rgba(53, 208, 127, 0.8);
  flex-shrink: 0;
}

.main {
  position: relative;
  z-index: 1;
  flex: 1;
  overflow-y: auto;
  padding: 28px 32px 36px;
}
</style>
