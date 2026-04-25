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
import { Separator } from '@/components/ui/separator'
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
  profile.value = await hugsStore.getUserProfile(userId.value)
  cooldown.value = await hugsStore.getCooldown(userId.value)
}

onMounted(load)
</script>

<template>
  <div class="mx-auto max-w-2xl space-y-6">
    <div v-if="loading" class="space-y-4">
      <Skeleton class="h-32 w-full rounded-lg" />
      <div class="grid grid-cols-3 gap-4">
        <Skeleton class="h-20 rounded-lg" />
        <Skeleton class="h-20 rounded-lg" />
        <Skeleton class="h-20 rounded-lg" />
      </div>
    </div>

    <div v-else-if="error" class="py-12 text-center text-muted-foreground">{{ error }}</div>

    <template v-else-if="profile">
      <!-- Profile header -->
      <Card>
        <CardContent class="flex items-center gap-5 p-4">
          <Avatar class="size-16">
            <AvatarFallback class="text-lg">
              {{ profile.username.slice(0, 2).toUpperCase() }}
            </AvatarFallback>
          </Avatar>
          <div class="flex-1 space-y-1.5">
            <h1 class="text-xl font-semibold">{{ profile.username }}</h1>
            <div class="flex items-center gap-2">
              <RankBadge :rank="profile.rank" />
              <span class="text-xs text-muted-foreground">
                {{ profile.role === 'admin' ? 'Администратор' : 'Пользователь' }}
              </span>
            </div>
          </div>
          <HugButton v-if="!isMe" :userId="userId" size="lg" @hugged="onHugged" />
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
          <div class="flex items-center justify-between">
            <p class="text-sm text-muted-foreground">
              <Coins class="inline size-3.5 mr-1" />
              5 монет = -10 мин. (минимум 5 мин.)
            </p>
            <Button
              @click="upgrade"
              :disabled="upgrading || cooldown.cooldown_seconds <= 300"
              variant="outline"
              size="sm"
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
