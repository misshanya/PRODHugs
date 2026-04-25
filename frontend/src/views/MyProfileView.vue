<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useAuthStore } from '@/stores/auth'
import { useHugsStore, type UserProfile } from '@/stores/hugs'
import RankBadge from '@/components/RankBadge.vue'

const auth = useAuthStore()
const hugsStore = useHugsStore()

const profile = ref<UserProfile | null>(null)
const history = ref<any[]>([])
const loading = ref(true)

onMounted(async () => {
  if (!auth.user) return
  loading.value = true
  try {
    profile.value = await hugsStore.getUserProfile(auth.user.id)
    history.value = await hugsStore.getHugHistory()
  } finally {
    loading.value = false
  }
})

function formatDate(dateStr: string): string {
  const d = new Date(dateStr)
  return d.toLocaleString('ru-RU', {
    day: 'numeric',
    month: 'short',
    hour: '2-digit',
    minute: '2-digit',
  })
}
</script>

<template>
  <div class="max-w-3xl mx-auto">
    <h1 class="text-2xl font-bold mb-6">👤 Мой профиль</h1>

    <div v-if="loading" class="text-center py-8 text-indigo-400">Загрузка...</div>

    <div v-else-if="profile" class="space-y-6">
      <!-- Profile card -->
      <div class="card">
        <div class="flex items-center gap-6">
          <div class="w-20 h-20 rounded-full bg-gradient-to-br from-primary to-accent flex items-center justify-center text-3xl font-bold">
            {{ profile.username[0]?.toUpperCase() }}
          </div>
          <div>
            <h2 class="text-2xl font-bold">{{ profile.username }}</h2>
            <div class="flex items-center gap-3 mt-2">
              <RankBadge :rank="profile.rank" size="lg" />
            </div>
          </div>
        </div>
      </div>

      <!-- Stats -->
      <div class="grid grid-cols-3 gap-4">
        <div class="card text-center">
          <p class="text-2xl font-bold text-primary-light">{{ profile.total_hugs }}</p>
          <p class="text-xs text-indigo-400 mt-1">Всего</p>
        </div>
        <div class="card text-center">
          <p class="text-2xl font-bold text-pink-400">{{ profile.hugs_given }}</p>
          <p class="text-xs text-indigo-400 mt-1">Отправлено</p>
        </div>
        <div class="card text-center">
          <p class="text-2xl font-bold text-green-400">{{ profile.hugs_received }}</p>
          <p class="text-xs text-indigo-400 mt-1">Получено</p>
        </div>
      </div>

      <!-- Hug history -->
      <div class="card">
        <h3 class="font-semibold mb-4">📜 История объятий</h3>
        <div v-if="history.length === 0" class="text-indigo-400 text-sm">
          Пока нет объятий. Найдите кого-нибудь и обнимите!
        </div>
        <div v-else class="space-y-2 max-h-96 overflow-y-auto">
          <div
            v-for="hug in history"
            :key="hug.id"
            class="flex items-center justify-between py-2 px-3 rounded-lg bg-surface-light/50"
          >
            <div class="flex items-center gap-2">
              <span v-if="hug.giver_id === auth.user?.id">🤗→</span>
              <span v-else>←🤗</span>
              <span class="text-sm">
                <span v-if="hug.giver_id === auth.user?.id" class="text-pink-400">Вы обняли</span>
                <span v-else class="text-green-400">Вас обнял(а)</span>
              </span>
            </div>
            <span class="text-xs text-indigo-400">{{ formatDate(hug.created_at) }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
