<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { ArrowUp, ArrowDown, Heart } from 'lucide-vue-next'
import { useAuthStore } from '@/stores/auth'
import { useHugsStore, type UserProfile } from '@/stores/hugs'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Avatar, AvatarFallback } from '@/components/ui/avatar'
import { Separator } from '@/components/ui/separator'
import { Skeleton } from '@/components/ui/skeleton'
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
  return new Date(dateStr).toLocaleString('ru-RU', {
    day: 'numeric',
    month: 'short',
    hour: '2-digit',
    minute: '2-digit',
  })
}
</script>

<template>
  <div class="mx-auto max-w-2xl space-y-6">
    <div>
      <h1 class="text-2xl font-semibold tracking-tight">Мой профиль</h1>
      <p class="text-muted-foreground">Ваша статистика и история обнимашек</p>
    </div>

    <div v-if="loading" class="space-y-4">
      <Skeleton class="h-28 w-full rounded-lg" />
      <div class="grid grid-cols-3 gap-4">
        <Skeleton class="h-20 rounded-lg" />
        <Skeleton class="h-20 rounded-lg" />
        <Skeleton class="h-20 rounded-lg" />
      </div>
    </div>

    <template v-else-if="profile">
      <!-- Profile card -->
      <Card>
        <CardContent class="flex items-center gap-5 p-4">
          <Avatar class="size-16">
            <AvatarFallback class="text-lg">
              {{ profile.username.slice(0, 2).toUpperCase() }}
            </AvatarFallback>
          </Avatar>
          <div class="space-y-1.5">
            <h2 class="text-xl font-semibold">{{ profile.username }}</h2>
            <RankBadge :rank="profile.rank" />
          </div>
        </CardContent>
      </Card>

      <!-- Stats -->
      <div class="grid grid-cols-3 gap-4">
        <Card>
          <CardHeader class="flex flex-row items-center justify-between pb-2">
            <CardDescription>Всего</CardDescription>
            <Heart class="size-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div class="text-2xl font-bold">{{ profile.total_hugs }}</div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader class="flex flex-row items-center justify-between pb-2">
            <CardDescription>Отправлено</CardDescription>
            <ArrowUp class="size-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div class="text-2xl font-bold">{{ profile.hugs_given }}</div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader class="flex flex-row items-center justify-between pb-2">
            <CardDescription>Получено</CardDescription>
            <ArrowDown class="size-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div class="text-2xl font-bold">{{ profile.hugs_received }}</div>
          </CardContent>
        </Card>
      </div>

      <!-- History -->
      <Card>
        <CardHeader>
          <CardTitle class="text-base">История обнимашек</CardTitle>
          <CardDescription>Последние обнимашки</CardDescription>
        </CardHeader>
        <CardContent>
          <div v-if="history.length === 0" class="py-6 text-center text-sm text-muted-foreground">
            Пока нет обнимашек
          </div>
          <div v-else class="space-y-1 max-h-96 overflow-y-auto">
            <div
              v-for="(hug, i) in history"
              :key="hug.id"
            >
              <Separator v-if="i > 0" class="my-1" />
              <div class="flex items-center justify-between py-2">
                <div class="flex items-center gap-2 text-sm">
                  <ArrowUp v-if="hug.giver_id === auth.user?.id" class="size-3.5 text-muted-foreground" />
                  <ArrowDown v-else class="size-3.5 text-muted-foreground" />
                  <span v-if="hug.giver_id === auth.user?.id" class="text-muted-foreground">
                    Ты обнял(а) кого-то
                  </span>
                  <span v-else class="text-muted-foreground">
                    Тебя кто-то обнял(а)
                  </span>
                </div>
                <span class="text-xs text-muted-foreground tabular-nums">
                  {{ formatDate(hug.created_at) }}
                </span>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>
    </template>
  </div>
</template>
