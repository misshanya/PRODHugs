<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, nextTick } from 'vue'
import { Search, Loader2 } from 'lucide-vue-next'
import { useHugsStore } from '@/stores/hugs'
import { useOnlineStore } from '@/stores/online'
import { Input } from '@/components/ui/input'
import { Skeleton } from '@/components/ui/skeleton'
import UserCard from '@/components/UserCard.vue'
import OutgoingHugsSection from '@/components/OutgoingHugsSection.vue'

const PAGE_SIZE = 30

const hugsStore = useHugsStore()
const onlineStore = useOnlineStore()
const query = ref('')
const users = ref<any[]>([])
const loading = ref(false)
const loadingMore = ref(false)
const hasMore = ref(true)
const sentinel = ref<HTMLElement | null>(null)

// ── Sorted users: online first, backend order preserved within each group ──
const sortedUsers = computed(() =>
  [...users.value].sort((a, b) => {
    const aOnline = onlineStore.isOnline(a.id) ? 0 : 1
    const bOnline = onlineStore.isOnline(b.id) ? 0 : 1
    return aOnline - bOnline
  }),
)

let debounceTimer: ReturnType<typeof setTimeout> | null = null
// Monotonic counter to discard out-of-order search responses.
let searchGeneration = 0
let observer: IntersectionObserver | null = null

async function search() {
  const gen = ++searchGeneration
  loading.value = true
  hasMore.value = true
  try {
    const result = await hugsStore.searchUsers(query.value, PAGE_SIZE, 0)
    if (gen !== searchGeneration) return
    users.value = result
    hasMore.value = result.length >= PAGE_SIZE
  } finally {
    if (gen === searchGeneration) {
      loading.value = false
    }
  }
  await nextTick()
  observeSentinel()
}

async function loadMore() {
  if (loadingMore.value || !hasMore.value || loading.value) return
  const gen = searchGeneration
  loadingMore.value = true
  try {
    const result = await hugsStore.searchUsers(query.value, PAGE_SIZE, users.value.length)
    if (gen !== searchGeneration) return
    users.value.push(...result)
    hasMore.value = result.length >= PAGE_SIZE
  } finally {
    if (gen === searchGeneration) {
      loadingMore.value = false
    }
  }
}

function observeSentinel() {
  observer?.disconnect()
  if (!sentinel.value) return
  observer = new IntersectionObserver(
    (entries) => {
      if (entries[0]?.isIntersecting) {
        loadMore()
      }
    },
    { rootMargin: '200px' },
  )
  observer.observe(sentinel.value)
}

watch(query, () => {
  if (debounceTimer) clearTimeout(debounceTimer)
  debounceTimer = setTimeout(search, 300)
})

onMounted(search)

onUnmounted(() => {
  if (debounceTimer) {
    clearTimeout(debounceTimer)
    debounceTimer = null
  }
  observer?.disconnect()
  // Increment generation so any in-flight search response is discarded.
  searchGeneration++
})
</script>

<template>
  <div class="mx-auto max-w-2xl space-y-6">
    <div>
      <h1 class="text-2xl font-semibold tracking-tight">Пользователи</h1>
      <p class="text-muted-foreground">Обнимись с кем-нибудь</p>
    </div>

    <OutgoingHugsSection />

    <div class="relative">
      <Search class="absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
      <Input
        v-model="query"
        type="text"
        class="pl-9"
        placeholder="Поиск по имени..."
        maxlength="64"
      />
    </div>

    <div v-if="loading" class="space-y-3">
      <Skeleton v-for="i in 5" :key="i" class="h-16 w-full rounded-lg" />
    </div>

    <div v-else-if="users.length === 0" class="py-12 text-center text-muted-foreground">
      Такого не нашли(
    </div>

    <div v-else class="space-y-2">
      <TransitionGroup name="user-list" tag="div" class="space-y-2">
        <UserCard v-for="user in sortedUsers" :key="user.id" :user="user" />
      </TransitionGroup>

      <div v-if="hasMore" ref="sentinel" class="flex justify-center py-4">
        <Loader2 v-if="loadingMore" class="size-5 animate-spin text-muted-foreground" />
      </div>
    </div>
  </div>
</template>

<style scoped>
.user-list-move {
  transition: transform 0.4s ease;
}
</style>
