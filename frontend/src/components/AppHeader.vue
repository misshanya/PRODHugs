<script setup lang="ts">
import { useAuthStore } from '@/stores/auth'
import { useHugsStore } from '@/stores/hugs'
import { onMounted } from 'vue'

const auth = useAuthStore()
const hugs = useHugsStore()

onMounted(() => {
  hugs.fetchBalance()
})
</script>

<template>
  <header class="fixed top-0 left-0 right-0 h-16 bg-surface border-b border-indigo-800/30 z-50">
    <div class="flex items-center justify-between h-full px-6">
      <div class="flex items-center gap-3">
        <span class="text-2xl">🤗</span>
        <h1 class="text-xl font-bold bg-gradient-to-r from-primary-light to-accent bg-clip-text text-transparent">
          Hugs as a Service
        </h1>
      </div>

      <div class="flex items-center gap-6">
        <div class="flex items-center gap-2 bg-surface-light rounded-xl px-4 py-2">
          <span class="text-lg">💰</span>
          <span class="font-semibold text-primary-light">{{ hugs.balance?.amount ?? 0 }}</span>
          <span class="text-xs text-indigo-400">монет</span>
        </div>

        <div class="flex items-center gap-3">
          <div class="w-8 h-8 rounded-full bg-primary flex items-center justify-center text-sm font-bold">
            {{ auth.user?.username?.[0]?.toUpperCase() }}
          </div>
          <span class="text-sm font-medium">{{ auth.user?.username }}</span>
        </div>

        <button
          @click="auth.logout()"
          class="text-sm text-indigo-400 hover:text-white transition-colors"
        >
          Выйти
        </button>
      </div>
    </div>
  </header>
</template>
