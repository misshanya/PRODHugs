<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { Wifi, WifiOff } from 'lucide-vue-next'
import { useHugsStore, type HugFeedItem } from '@/stores/hugs'
import { Badge } from '@/components/ui/badge'
import { Separator } from '@/components/ui/separator'
import { Skeleton } from '@/components/ui/skeleton'

const hugsStore = useHugsStore()
const feed = ref<HugFeedItem[]>([])
const connected = ref(false)
const initialLoading = ref(true)
let ws: WebSocket | null = null

function timeAgo(dateStr: string): string {
  const diff = Math.floor((Date.now() - new Date(dateStr).getTime()) / 1000)
  if (diff < 60) return `${diff} сек.`
  if (diff < 3600) return `${Math.floor(diff / 60)} мин.`
  if (diff < 86400) return `${Math.floor(diff / 3600)} ч.`
  return new Date(dateStr).toLocaleDateString('ru-RU', { day: 'numeric', month: 'short' })
}

function connectWS() {
  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
  ws = new WebSocket(`${protocol}//${window.location.host}/api/v1/ws`)

  ws.onopen = () => {
    connected.value = true
  }

  ws.onmessage = (event) => {
    try {
      const item = JSON.parse(event.data) as HugFeedItem
      feed.value.unshift(item)
      if (feed.value.length > 100) feed.value = feed.value.slice(0, 100)
    } catch {
      // Ignore
    }
  }

  ws.onclose = () => {
    connected.value = false
    setTimeout(connectWS, 3000)
  }

  ws.onerror = () => {
    ws?.close()
  }
}

onMounted(async () => {
  await hugsStore.fetchFeed(50)
  feed.value = [...hugsStore.feed]
  initialLoading.value = false
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
  <div class="mx-auto max-w-2xl space-y-6">
    <div class="flex items-center justify-between">
      <div>
        <h1 class="text-2xl font-semibold tracking-tight">Лента</h1>
        <p class="text-muted-foreground">Обнимашки в реальном времени</p>
      </div>
      <Badge :variant="connected ? 'secondary' : 'destructive'" class="gap-1.5">
        <Wifi v-if="connected" class="size-3" />
        <WifiOff v-else class="size-3" />
        {{ connected ? 'Подключено' : 'Отключено' }}
      </Badge>
    </div>

    <div v-if="initialLoading" class="space-y-3">
      <Skeleton v-for="i in 8" :key="i" class="h-12 w-full" />
    </div>

    <div v-else-if="feed.length === 0" class="py-16 text-center text-muted-foreground">
      <p class="text-lg font-medium">Пока нет обнимашек</p>
      <p class="mt-1 text-sm">Будьте первыми!</p>
    </div>

    <div v-else class="rounded-md border divide-y">
      <TransitionGroup name="feed">
        <div
          v-for="item in feed"
          :key="item.id"
          class="flex items-center gap-3 px-4 py-3"
        >
          <div class="flex-1 min-w-0 text-sm">
            <RouterLink
              :to="`/user/${item.giver_id}`"
              class="font-medium hover:underline"
            >{{ item.giver_username }}</RouterLink>
            <span class="text-muted-foreground mx-1.5">обнял(а)</span>
            <RouterLink
              :to="`/user/${item.receiver_id}`"
              class="font-medium hover:underline"
            >{{ item.receiver_username }}</RouterLink>
          </div>
          <span class="shrink-0 text-xs text-muted-foreground tabular-nums">
            {{ timeAgo(item.created_at) }}
          </span>
        </div>
      </TransitionGroup>
    </div>
  </div>
</template>

<style scoped>
.feed-enter-active {
  transition: all 0.3s ease;
}
.feed-enter-from {
  opacity: 0;
  transform: translateY(-10px);
}
</style>
