<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRoute } from 'vue-router'
import { useHugsStore, type UserProfile, type CooldownInfo } from '@/stores/hugs'
import { useAuthStore } from '@/stores/auth'
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
  } catch (e: any) {
    alert(e.response?.data?.message || 'Недостаточно монет')
  } finally {
    upgrading.value = false
  }
}

async function onHugged() {
  // Reload profile to update stats
  profile.value = await hugsStore.getUserProfile(userId.value)
}

onMounted(load)
</script>

<template>
  <div class="max-w-2xl mx-auto">
    <div v-if="loading" class="text-center py-12 text-indigo-400">Загрузка...</div>
    <div v-else-if="error" class="text-center py-12 text-red-400">{{ error }}</div>
    <div v-else-if="profile" class="space-y-6">
      <!-- Profile header -->
      <div class="card">
        <div class="flex items-center gap-6">
          <div class="w-20 h-20 rounded-full bg-gradient-to-br from-primary to-accent flex items-center justify-center text-3xl font-bold shrink-0">
            {{ profile.username[0]?.toUpperCase() }}
          </div>
          <div class="flex-1">
            <h1 class="text-2xl font-bold">{{ profile.username }}</h1>
            <div class="flex items-center gap-3 mt-2">
              <RankBadge :rank="profile.rank" />
              <span class="text-sm text-indigo-400">
                {{ profile.role === 'admin' ? 'Администратор' : 'Пользователь' }}
              </span>
            </div>
          </div>
          <HugButton v-if="!isMe" :userId="userId" @hugged="onHugged" />
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

      <!-- Cooldown upgrade (only for other users) -->
      <div v-if="!isMe && cooldown" class="card">
        <h3 class="font-semibold mb-3">⏱️ Кулдаун для этого пользователя</h3>
        <div class="flex items-center justify-between">
          <div>
            <p class="text-sm text-indigo-400">
              Текущий кулдаун: <span class="text-white font-semibold">{{ Math.floor(cooldown.cooldown_seconds / 60) }} мин.</span>
            </p>
            <p class="text-xs text-indigo-500 mt-1">
              Стоимость улучшения: 5 монет (-10 мин., мин. 5 мин.)
            </p>
          </div>
          <button
            @click="upgrade"
            :disabled="upgrading || cooldown.cooldown_seconds <= 300"
            :class="[
              'btn-primary text-sm',
              cooldown.cooldown_seconds <= 300 ? 'opacity-50 cursor-not-allowed' : '',
            ]"
          >
            {{ upgrading ? '...' : cooldown.cooldown_seconds <= 300 ? 'Макс.' : '⬆️ Улучшить' }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
