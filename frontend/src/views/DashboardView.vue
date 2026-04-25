<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useAuthStore } from '@/stores/auth'
import { useHugsStore, type DailyRewardResponse, type UserProfile } from '@/stores/hugs'
import RankBadge from '@/components/RankBadge.vue'

const auth = useAuthStore()
const hugs = useHugsStore()

const profile = ref<UserProfile | null>(null)
const dailyResult = ref<DailyRewardResponse | null>(null)
const claimingDaily = ref(false)
const dailyError = ref('')

onMounted(async () => {
  await hugs.fetchBalance()
  if (auth.user) {
    profile.value = await hugs.getUserProfile(auth.user.id)
  }
})

async function claimDaily() {
  claimingDaily.value = true
  dailyError.value = ''
  try {
    dailyResult.value = await hugs.claimDailyReward()
  } catch (e: any) {
    dailyError.value = e.response?.data?.message || 'Ошибка'
  } finally {
    claimingDaily.value = false
  }
}
</script>

<template>
  <div class="max-w-4xl mx-auto space-y-6">
    <div class="flex items-center gap-4 mb-2">
      <span class="text-4xl">👋</span>
      <div>
        <h1 class="text-2xl font-bold">Привет, {{ auth.user?.username }}!</h1>
        <p class="text-indigo-400">Добро пожаловать в мир объятий</p>
      </div>
    </div>

    <!-- Stats cards -->
    <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
      <div class="card text-center">
        <p class="text-3xl font-bold text-primary-light">{{ profile?.total_hugs ?? 0 }}</p>
        <p class="text-sm text-indigo-400 mt-1">Всего объятий</p>
      </div>
      <div class="card text-center">
        <p class="text-3xl font-bold text-pink-400">{{ profile?.hugs_given ?? 0 }}</p>
        <p class="text-sm text-indigo-400 mt-1">Отправлено</p>
      </div>
      <div class="card text-center">
        <p class="text-3xl font-bold text-green-400">{{ profile?.hugs_received ?? 0 }}</p>
        <p class="text-sm text-indigo-400 mt-1">Получено</p>
      </div>
    </div>

    <!-- Rank & Balance -->
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      <div class="card">
        <h3 class="text-lg font-semibold mb-3">Ваш ранг</h3>
        <div class="flex items-center gap-4">
          <RankBadge :rank="profile?.rank ?? 'Новичок'" size="lg" />
          <div class="text-sm text-indigo-400">
            <p>Набирайте объятия чтобы повысить ранг!</p>
            <p class="mt-1">Следующий ранг при большем количестве hugs.</p>
          </div>
        </div>
      </div>

      <div class="card">
        <h3 class="text-lg font-semibold mb-3">Баланс</h3>
        <div class="flex items-center justify-between">
          <div>
            <p class="text-3xl font-bold text-amber-400">
              💰 {{ hugs.balance?.amount ?? 0 }}
            </p>
            <p class="text-sm text-indigo-400 mt-1">монет на счету</p>
          </div>
        </div>
      </div>
    </div>

    <!-- Daily reward -->
    <div class="card">
      <h3 class="text-lg font-semibold mb-3">🎁 Ежедневная награда</h3>
      <div class="flex items-center justify-between">
        <div>
          <p class="text-sm text-indigo-400">
            Заходите каждый день, чтобы получить бонус! Серия дней увеличивает награду.
          </p>
          <p v-if="dailyResult" class="mt-2">
            <span v-if="dailyResult.already_claimed" class="text-yellow-400">
              ✅ Сегодня уже получено! Серия: {{ dailyResult.streak_days }} дн.
            </span>
            <span v-else class="text-green-400">
              🎉 Получено +{{ dailyResult.amount }} монет! Серия: {{ dailyResult.streak_days }} дн. Баланс: {{ dailyResult.new_balance }}
            </span>
          </p>
          <p v-if="dailyError" class="text-red-400 text-sm mt-2">{{ dailyError }}</p>
        </div>
        <button
          @click="claimDaily"
          :disabled="claimingDaily"
          class="btn-accent shrink-0"
        >
          {{ claimingDaily ? '...' : 'Забрать' }}
        </button>
      </div>
    </div>

    <!-- Quick links -->
    <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
      <RouterLink to="/users" class="card hover:border-primary/50 transition-all text-center group">
        <span class="text-3xl">👥</span>
        <p class="font-semibold mt-2 group-hover:text-primary-light">Найти пользователей</p>
        <p class="text-xs text-indigo-400 mt-1">Обнимите кого-нибудь!</p>
      </RouterLink>
      <RouterLink to="/feed" class="card hover:border-primary/50 transition-all text-center group">
        <span class="text-3xl">📢</span>
        <p class="font-semibold mt-2 group-hover:text-primary-light">Лента объятий</p>
        <p class="text-xs text-indigo-400 mt-1">Все объятия в реальном времени</p>
      </RouterLink>
      <RouterLink to="/leaderboard" class="card hover:border-primary/50 transition-all text-center group">
        <span class="text-3xl">🏆</span>
        <p class="font-semibold mt-2 group-hover:text-primary-light">Рейтинг</p>
        <p class="text-xs text-indigo-400 mt-1">Топ обнимателей</p>
      </RouterLink>
    </div>
  </div>
</template>
