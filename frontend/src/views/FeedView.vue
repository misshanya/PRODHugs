<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { useHugsStore, type HugFeedItem } from '@/stores/hugs'

const hugsStore = useHugsStore()
const feed = ref<HugFeedItem[]>([])
const connected = ref(false)
let ws: WebSocket | null = null

function formatDate(dateStr: string): string {
  const d = new Date(dateStr)
  return d.toLocaleString('ru-RU', {
    day: 'numeric',
    month: 'short',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  })
}

function timeAgo(dateStr: string): string {
  const now = Date.now()
  const d = new Date(dateStr).getTime()
  const diff = Math.floor((now - d) / 1000)

  if (diff < 60) return `${diff} сек. назад`
  if (diff < 3600) return `${Math.floor(diff / 60)} мин. назад`
  if (diff < 86400) return `${Math.floor(diff / 3600)} ч. назад`
  return formatDate(dateStr)
}

function connectWS() {
  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
  const host = window.location.host
  ws = new WebSocket(`${protocol}//${host}/api/v1/ws`)

  ws.onopen = () => {
    connected.value = true
  }

  ws.onmessage = (event) => {
    try {
      const item = JSON.parse(event.data) as HugFeedItem
      feed.value.unshift(item)
      // Keep max 100 items
      if (feed.value.length > 100) {
        feed.value = feed.value.slice(0, 100)
      }
    } catch {
      // Ignore malformed messages
    }
  }

  ws.onclose = () => {
    connected.value = false
    // Reconnect after 3 seconds
    setTimeout(connectWS, 3000)
  }

  ws.onerror = () => {
    ws?.close()
  }
}

onMounted(async () => {
  // Load initial feed
  await hugsStore.fetchFeed(50)
  feed.value = [...hugsStore.feed]

  // Connect WebSocket for real-time updates
  connectWS()
})

onUnmounted(() => {
  if (ws) {
    ws.close()
    ws = null
  }
})
</script>

<template>
  <div class="max-w-3xl mx-auto">
    <div class="flex items-center justify-between mb-6">
      <h1 class="text-2xl font-bold">📢 Лента объятий</h1>
      <div class="flex items-center gap-2">
        <span
          :class="['w-2 h-2 rounded-full', connected ? 'bg-green-500' : 'bg-red-500']"
        ></span>
        <span class="text-xs text-indigo-400">
          {{ connected ? 'Подключено' : 'Переподключение...' }}
        </span>
      </div>
    </div>

    <div v-if="feed.length === 0" class="text-center py-12 text-indigo-400">
      <span class="text-4xl">🤗</span>
      <p class="mt-4">Пока нет объятий. Будьте первыми!</p>
    </div>

    <TransitionGroup name="feed" tag="div" class="space-y-3">
      <div
        v-for="item in feed"
        :key="item.id"
        class="card flex items-center gap-4"
      >
        <span class="text-2xl shrink-0">🤗</span>
        <div class="flex-1 min-w-0">
          <p>
            <RouterLink
              :to="`/user/${item.giver_id}`"
              class="font-semibold text-pink-400 hover:underline"
            >
              {{ item.giver_username }}
            </RouterLink>
            <span class="text-indigo-400"> обнял(а) </span>
            <RouterLink
              :to="`/user/${item.receiver_id}`"
              class="font-semibold text-green-400 hover:underline"
            >
              {{ item.receiver_username }}
            </RouterLink>
          </p>
        </div>
        <span class="text-xs text-indigo-500 shrink-0">{{ timeAgo(item.created_at) }}</span>
      </div>
    </TransitionGroup>
  </div>
</template>

<style scoped>
.feed-enter-active {
  transition: all 0.4s ease;
}

.feed-enter-from {
  opacity: 0;
  transform: translateY(-20px);
}

.feed-leave-active {
  transition: all 0.3s ease;
}

.feed-leave-to {
  opacity: 0;
}
</style>
