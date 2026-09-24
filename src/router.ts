import { createRouter, createWebHashHistory } from 'vue-router'
import GenerateView from './views/GenerateView.vue'

export default createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: '/', redirect: '/generate' },
    { path: '/generate', component: GenerateView },
    { path: '/library', component: () => import('./views/LibraryView.vue') },
    { path: '/settings', component: () => import('./views/SettingsView.vue') },
  ],
})
