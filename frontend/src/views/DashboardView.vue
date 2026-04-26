<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { Heart, ArrowUp, ArrowDown, Gift, Coins, Users, Trophy, Newspaper } from 'lucide-vue-next'
import { toast } from 'vue-sonner'
import { useAuthStore } from '@/stores/auth'
import { useHugsStore, type DailyRewardResponse, type UserProfile, type HugFeedItem } from '@/stores/hugs'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Separator } from '@/components/ui/separator'
import RankBadge from '@/components/RankBadge.vue'
import { Progress } from '@/components/ui/progress'
import { Skeleton } from '@/components/ui/skeleton'
import { plural, hugVerb } from '@/lib/utils'

const auth = useAuthStore()
const hugs = useHugsStore()

const profile = ref<UserProfile | null>(null)
const history = ref<HugFeedItem[]>([])
const dailyResult = ref<DailyRewardResponse | null>(null)
const claimingDaily = ref(false)
const loading = ref(true)

const rankThresholds = [
  { name: 'Новичок', min: 0 },
  { name: 'Обнимашка', min: 10 },
  { name: 'Дружелюбный', min: 50 },
  { name: 'Мастер обнимашек', min: 200 },
  { name: 'Легенда', min: 500 },
  { name: 'Бог обнимашек', min: 1000 },
]

function getRankProgress(totalHugs: number) {
  const currentIdx = rankThresholds.findLastIndex((r: { name: string; min: number }) => totalHugs >= r.min)
  const nextIdx = currentIdx + 1
  if (nextIdx >= rankThresholds.length) return { progress: 100, nextRank: null, needed: 0 }
  const current = rankThresholds[currentIdx]!
  const next = rankThresholds[nextIdx]!
  const progress = ((totalHugs - current.min) / (next.min - current.min)) * 100
  return { progress: Math.min(progress, 100), nextRank: next.name, needed: next.min - totalHugs }
}

onMounted(async () => {
  await hugs.fetchBalance()
  if (auth.user) {
    const [p, h] = await Promise.all([
      hugs.getUserProfile(auth.user.id),
      hugs.getHugHistory(),
    ])
    profile.value = p
    history.value = h
  }
  loading.value = false
})

function formatDate(dateStr: string): string {
  return new Date(dateStr).toLocaleString('ru-RU', {
    day: 'numeric',
    month: 'short',
    hour: '2-digit',
    minute: '2-digit',
  })
}

async function claimDaily() {
  claimingDaily.value = true
  try {
    dailyResult.value = await hugs.claimDailyReward()
    if (dailyResult.value.already_claimed) {
      toast.info('Вы уже получили награду сегодня')
    } else {
      toast.success(`Получено +${plural(dailyResult.value.amount, 'монета', 'монеты', 'монет')}!`)
    }
  } catch (e: any) {
    toast.error(e.response?.data?.message || 'Ошибка')
  } finally {
    claimingDaily.value = false
  }
}

const rankInfo = () => getRankProgress(profile.value?.total_hugs ?? 0)
</script>

<template>
  <div class="mx-auto max-w-4xl space-y-6">
    <div>
      <h1 class="text-2xl font-semibold tracking-tight">
        Привет, {{ auth.user?.username }}
      </h1>
      <p class="text-muted-foreground">Ваша панель управления обнимашками</p>
    </div>

    <!-- Stats -->
    <div class="grid gap-4 sm:grid-cols-3">
      <Card v-if="!loading">
        <CardHeader class="flex flex-row items-center justify-between pb-2">
          <CardDescription>Всего обнимашек</CardDescription>
          <Heart class="size-4 text-muted-foreground" />
        </CardHeader>
        <CardContent>
          <div class="text-2xl font-bold">{{ profile?.total_hugs ?? 0 }}</div>
        </CardContent>
      </Card>
      <Card v-if="!loading">
        <CardHeader class="flex flex-row items-center justify-between pb-2">
          <CardDescription>Отправлено</CardDescription>
          <ArrowUp class="size-4 text-muted-foreground" />
        </CardHeader>
        <CardContent>
          <div class="text-2xl font-bold">{{ profile?.hugs_given ?? 0 }}</div>
        </CardContent>
      </Card>
      <Card v-if="!loading">
        <CardHeader class="flex flex-row items-center justify-between pb-2">
          <CardDescription>Получено</CardDescription>
          <ArrowDown class="size-4 text-muted-foreground" />
        </CardHeader>
        <CardContent>
          <div class="text-2xl font-bold">{{ profile?.hugs_received ?? 0 }}</div>
        </CardContent>
      </Card>
      <template v-if="loading">
        <Card v-for="i in 3" :key="i">
          <CardHeader class="pb-2"><Skeleton class="h-4 w-24" /></CardHeader>
          <CardContent><Skeleton class="h-8 w-16" /></CardContent>
        </Card>
      </template>
    </div>

    <div class="grid gap-4 md:grid-cols-2">
      <!-- Rank & Progress -->
      <Card>
        <CardHeader>
          <CardTitle class="text-base">Ваш ранг</CardTitle>
        </CardHeader>
        <CardContent class="space-y-4">
          <div class="flex items-center gap-3">
            <RankBadge :rank="profile?.rank ?? 'Новичок'" />
            <div class="flex items-center gap-1.5 text-sm text-muted-foreground">
              <Coins class="size-3.5" />
              {{ plural(hugs.balance?.amount ?? 0, 'монета', 'монеты', 'монет') }}
            </div>
          </div>
          <div v-if="rankInfo().nextRank" class="space-y-2">
            <div class="flex justify-between text-xs text-muted-foreground">
              <span>{{ profile?.rank }}</span>
              <span>{{ rankInfo().nextRank }}</span>
            </div>
            <Progress :model-value="rankInfo().progress" class="h-2" />
            <p class="text-xs text-muted-foreground">
              Ещё {{ plural(rankInfo().needed, 'обнимашка', 'обнимашки', 'обнимашек') }} до следующего ранга
            </p>
          </div>
          <p v-else class="text-xs text-muted-foreground">Максимальный ранг достигнут</p>
        </CardContent>
      </Card>

      <!-- Daily reward -->
      <Card>
        <CardHeader>
          <CardTitle class="text-base">Ежедневная награда</CardTitle>
          <CardDescription>Заходите каждый день для бонуса. Серия увеличивает награду.</CardDescription>
        </CardHeader>
        <CardContent class="space-y-3">
          <div v-if="dailyResult" class="text-sm">
            <p v-if="dailyResult.already_claimed" class="text-muted-foreground">
              Уже получено сегодня. Серия: {{ dailyResult.streak_days }} дн.
            </p>
            <p v-else class="text-green-400">
              +{{ plural(dailyResult.amount, 'монета', 'монеты', 'монет') }}! Серия: {{ dailyResult.streak_days }} дн.
            </p>
          </div>
          <Button @click="claimDaily" :disabled="claimingDaily" variant="outline" class="w-full">
            <Gift class="size-4" />
            {{ claimingDaily ? 'Загрузка...' : 'Забрать награду' }}
          </Button>
        </CardContent>
      </Card>
    </div>

    <!-- Hug history -->
    <Card>
      <CardHeader>
        <CardTitle class="text-base">История обнимашек</CardTitle>
        <CardDescription>Последние обнимашки</CardDescription>
      </CardHeader>
      <CardContent>
        <div v-if="loading" class="space-y-3">
          <Skeleton v-for="i in 3" :key="i" class="h-8 w-full rounded" />
        </div>
        <div v-else-if="history.length === 0" class="py-6 text-center text-sm text-muted-foreground">
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
                  Ты {{ hugVerb(auth.user?.gender) }}
                  <RouterLink :to="`/user/${hug.receiver_id}`" class="text-foreground font-medium hover:underline">{{ hug.receiver_username }}</RouterLink>
                </span>
                <span v-else class="text-muted-foreground">
                  <RouterLink :to="`/user/${hug.giver_id}`" class="text-foreground font-medium hover:underline">{{ hug.giver_username }}</RouterLink>
                  {{ hugVerb(hug.giver_gender) }} тебя
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

    <!-- Quick links -->
    <div class="grid gap-4 sm:grid-cols-3">
      <RouterLink to="/users">
        <Card class="transition-colors hover:bg-accent/50 h-full">
          <CardHeader class="flex flex-row items-center gap-3">
            <Users class="size-5 text-muted-foreground" />
            <div>
              <CardTitle class="text-sm">Пользователи</CardTitle>
              <CardDescription>Найти и обнять</CardDescription>
            </div>
          </CardHeader>
        </Card>
      </RouterLink>
      <RouterLink to="/feed">
        <Card class="transition-colors hover:bg-accent/50 h-full">
          <CardHeader class="flex flex-row items-center gap-3">
            <Newspaper class="size-5 text-muted-foreground" />
            <div>
              <CardTitle class="text-sm">Лента</CardTitle>
              <CardDescription>Обнимашки в реальном времени</CardDescription>
            </div>
          </CardHeader>
        </Card>
      </RouterLink>
      <RouterLink to="/leaderboard">
        <Card class="transition-colors hover:bg-accent/50 h-full">
          <CardHeader class="flex flex-row items-center gap-3">
            <Trophy class="size-5 text-muted-foreground" />
            <div>
              <CardTitle class="text-sm">Рейтинг</CardTitle>
              <CardDescription>Топ пользователей</CardDescription>
            </div>
          </CardHeader>
        </Card>
      </RouterLink>
    </div>
  </div>
</template>
