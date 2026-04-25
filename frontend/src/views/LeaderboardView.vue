<script setup lang="ts">
import { onMounted } from 'vue'
import { useHugsStore } from '@/stores/hugs'
import RankBadge from '@/components/RankBadge.vue'

const hugsStore = useHugsStore()

onMounted(() => {
  hugsStore.fetchLeaderboard(50, 0)
})
</script>

<template>
  <div class="max-w-3xl mx-auto">
    <h1 class="text-2xl font-bold mb-6">🏆 Рейтинг</h1>

    <div v-if="hugsStore.loading" class="text-center py-8 text-indigo-400">Загрузка...</div>

    <div v-else-if="hugsStore.leaderboard.length === 0" class="text-center py-8 text-indigo-400">
      Пока нет данных. Будьте первыми!
    </div>

    <div v-else class="space-y-3">
      <RouterLink
        v-for="(entry, index) in hugsStore.leaderboard"
        :key="entry.user_id"
        :to="`/user/${entry.user_id}`"
        class="card flex items-center gap-4 hover:border-primary/50 transition-all"
      >
        <div
          :class="[
            'w-10 h-10 rounded-full flex items-center justify-center font-bold text-lg shrink-0',
            index === 0 ? 'bg-amber-500/20 text-amber-400' :
            index === 1 ? 'bg-gray-400/20 text-gray-300' :
            index === 2 ? 'bg-orange-500/20 text-orange-400' :
            'bg-surface-light text-indigo-400',
          ]"
        >
          <span v-if="index === 0">🥇</span>
          <span v-else-if="index === 1">🥈</span>
          <span v-else-if="index === 2">🥉</span>
          <span v-else>{{ index + 1 }}</span>
        </div>

        <div class="flex-1 min-w-0">
          <p class="font-semibold truncate">{{ entry.username }}</p>
          <RankBadge :rank="entry.rank" size="sm" />
        </div>

        <div class="text-right shrink-0">
          <p class="text-lg font-bold text-primary-light">{{ entry.total_hugs }}</p>
          <p class="text-xs text-indigo-400">
            ↑{{ entry.hugs_given }} ↓{{ entry.hugs_received }}
          </p>
        </div>
      </RouterLink>
    </div>
  </div>
</template>
