<script setup lang="ts">
import { useRoute } from 'vue-router'
import { computed } from 'vue'

const route = useRoute()

const links = [
  { to: '/dashboard', label: 'Главная', icon: '🏠' },
  { to: '/users', label: 'Пользователи', icon: '👥' },
  { to: '/feed', label: 'Лента', icon: '📢' },
  { to: '/leaderboard', label: 'Рейтинг', icon: '🏆' },
  { to: '/profile', label: 'Мой профиль', icon: '👤' },
]

const currentPath = computed(() => route.path)
</script>

<template>
  <aside class="fixed left-0 top-16 bottom-0 w-64 bg-surface border-r border-indigo-800/30 p-4">
    <nav class="space-y-2 mt-4">
      <RouterLink
        v-for="link in links"
        :key="link.to"
        :to="link.to"
        :class="[
          'flex items-center gap-3 px-4 py-3 rounded-xl transition-all duration-200',
          currentPath === link.to || currentPath.startsWith(link.to + '/')
            ? 'bg-primary/20 text-primary-light border border-primary/30'
            : 'text-indigo-300 hover:bg-surface-light hover:text-white',
        ]"
      >
        <span class="text-lg">{{ link.icon }}</span>
        <span class="font-medium">{{ link.label }}</span>
      </RouterLink>
    </nav>
  </aside>
</template>
