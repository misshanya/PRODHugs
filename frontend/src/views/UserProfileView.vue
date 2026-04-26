<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRoute } from 'vue-router'
import { ArrowUp, ArrowDown, Heart, Coins, Clock, ArrowUpCircle } from 'lucide-vue-next'
import { toast } from 'vue-sonner'
import { useHugsStore, type UserProfile, type CooldownInfo } from '@/stores/hugs'
import { useAuthStore } from '@/stores/auth'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Avatar, AvatarFallback } from '@/components/ui/avatar'
import { Skeleton } from '@/components/ui/skeleton'
import HugButton from '@/components/HugButton.vue'
import RankBadge from '@/components/RankBadge.vue'

const route = useRoute()
const auth = useAuthStore()
const hugsStore = useHugsStore()

const profile = ref<UserProfile | null>(null)
const cooldown = ref<CooldownInfo | null>(null)
const loading = ref(true)
const upgrading = ref(false)
const error = ref('')

const userId = computed(() => route.params.id as string)
const isMe = computed(() => auth.user?.id === userId.value)

const prevStats = ref({ total: 0, given: 0, received: 0 })
const animatingStats = ref({ total: false, given: false, received: false })

function triggerStatsAnimation() {
  if (!profile.value) return
  const p = profile.value
  const changed = {
    total: p.total_hugs !== prevStats.value.total,
    given: p.hugs_given !== prevStats.value.given,
    received: p.hugs_received !== prevStats.value.received,
  }
  animatingStats.value = changed
  prevStats.value = { total: p.total_hugs, given: p.hugs_given, received: p.hugs_received }
  setTimeout(() => {
    animatingStats.value = { total: false, given: false, received: false }
  }, 600)
}

async function load() {
  loading.value = true
  try {
    profile.value = await hugsStore.getUserProfile(userId.value)
    if (!isMe.value) {
      cooldown.value = await hugsStore.getCooldown(userId.value)
    }
  } catch {
    error.value = 'Пользователь не найден'
  } finally {
    loading.value = false
  }
}

async function upgrade() {
  upgrading.value = true
  try {
    cooldown.value = await hugsStore.upgradeCooldown(userId.value)
    toast.success('Кулдаун уменьшен!')
  } catch (e: any) {
    toast.error(e.response?.data?.message || 'Недостаточно монет')
  } finally {
    upgrading.value = false
  }
}

async function onHugged() {
  if (profile.value) {
    prevStats.value = {
      total: profile.value.total_hugs,
      given: profile.value.hugs_given,
      received: profile.value.hugs_received,
    }
  }
  profile.value = await hugsStore.getUserProfile(userId.value)
  cooldown.value = await hugsStore.getCooldown(userId.value)
  triggerStatsAnimation()
}

onMounted(load)
</script>

<template>
  <div class="mx-auto max-w-2xl space-y-6">
    <div v-if="loading" class="space-y-4">
      <Skeleton class="h-40 w-full rounded-lg sm:h-32" />
      <div class="grid grid-cols-3 gap-2 sm:gap-4">
        <Skeleton class="h-16 rounded-lg sm:h-20" />
        <Skeleton class="h-16 rounded-lg sm:h-20" />
        <Skeleton class="h-16 rounded-lg sm:h-20" />
      </div>
    </div>

    <div v-else-if="error" class="py-12 text-center text-muted-foreground">{{ error }}</div>

    <template v-else-if="profile">
      <!-- Profile header -->
      <Card>
        <CardContent class="p-4">
          <!-- Mobile: stacked layout -->
          <div class="flex flex-col items-center gap-3 text-center sm:flex-row sm:items-center sm:gap-5 sm:text-left">
            <Avatar class="size-16 sm:size-16">
              <AvatarFallback class="text-lg">
                {{ profile.username.slice(0, 2).toUpperCase() }}
              </AvatarFallback>
            </Avatar>
            <div class="flex-1 space-y-1.5">
              <h1 class="text-lg font-semibold sm:text-xl">{{ profile.username }}</h1>
              <div class="flex items-center justify-center gap-2 sm:justify-start">
                <RankBadge :rank="profile.rank" />
                <span class="text-xs text-muted-foreground">
                  {{ profile.role === 'admin' ? 'Администратор' : 'Пользователь' }}
                </span>
              </div>
            </div>
            <HugButton v-if="!isMe" :userId="userId" :username="profile.username" size="lg" @hugged="onHugged" />
          </div>
        </CardContent>
      </Card>

      <!-- Stats -->
      <div class="grid grid-cols-3 gap-2 sm:gap-4">
        <Card>
          <CardHeader class="flex flex-row items-center justify-between px-3 pb-1 pt-3 sm:px-6 sm:pb-2 sm:pt-6">
            <CardDescription class="text-[11px] sm:text-sm">Всего</CardDescription>
            <Heart class="hidden size-4 text-prod-yellow sm:block" />
          </CardHeader>
          <CardContent class="px-3 pb-3 sm:px-6 sm:pb-6">
            <div class="text-xl font-bold tabular-nums sm:text-2xl" :class="animatingStats.total && 'stat-pop'">
              {{ profile.total_hugs }}
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader class="flex flex-row items-center justify-between px-3 pb-1 pt-3 sm:px-6 sm:pb-2 sm:pt-6">
            <CardDescription class="text-[11px] sm:text-sm">Отправлено</CardDescription>
            <ArrowUp class="hidden size-4 text-muted-foreground sm:block" />
          </CardHeader>
          <CardContent class="px-3 pb-3 sm:px-6 sm:pb-6">
            <div class="text-xl font-bold tabular-nums sm:text-2xl" :class="animatingStats.given && 'stat-pop'">
              {{ profile.hugs_given }}
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader class="flex flex-row items-center justify-between px-3 pb-1 pt-3 sm:px-6 sm:pb-2 sm:pt-6">
            <CardDescription class="text-[11px] sm:text-sm">Получено</CardDescription>
            <ArrowDown class="hidden size-4 text-muted-foreground sm:block" />
          </CardHeader>
          <CardContent class="px-3 pb-3 sm:px-6 sm:pb-6">
            <div class="text-xl font-bold tabular-nums sm:text-2xl" :class="animatingStats.received && 'stat-pop'">
              {{ profile.hugs_received }}
            </div>
          </CardContent>
        </Card>
      </div>

      <!-- Cooldown upgrade -->
      <Card v-if="!isMe && cooldown">
        <CardHeader>
          <CardTitle class="flex items-center gap-2 text-base">
            <Clock class="size-4" />
            Кулдаун
          </CardTitle>
          <CardDescription>
            Текущий кулдаун: {{ Math.floor(cooldown.cooldown_seconds / 60) }} мин.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div class="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
            <p class="text-xs text-muted-foreground sm:text-sm">
              <Coins class="inline size-3.5 mr-1" />
              5 монет = -10 мин. (мин. 5 мин.)
            </p>
            <Button
              @click="upgrade"
              :disabled="upgrading || cooldown.cooldown_seconds <= 300"
              variant="yellow"
              size="sm"
              class="w-full rounded-[21px] sm:w-auto"
            >
              <ArrowUpCircle class="size-4" />
              {{ cooldown.cooldown_seconds <= 300 ? 'Максимум' : 'Улучшить' }}
            </Button>
          </div>
        </CardContent>
      </Card>
    </template>
  </div>
</template>
