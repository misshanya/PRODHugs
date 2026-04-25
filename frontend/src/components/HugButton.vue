<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { useHugsStore, type CooldownInfo } from '@/stores/hugs'

const props = defineProps<{
  userId: string
  compact?: boolean
}>()

const emit = defineEmits<{
  hugged: []
}>()

const hugsStore = useHugsStore()
const loading = ref(false)
const cooldown = ref<CooldownInfo | null>(null)
const remaining = ref(0)
const animating = ref(false)
const showEmoji = ref(false)
let timer: ReturnType<typeof setInterval> | null = null

async function loadCooldown() {
  try {
    cooldown.value = await hugsStore.getCooldown(props.userId)
    remaining.value = cooldown.value.remaining_seconds
    startTimer()
  } catch {
    // Ignore
  }
}

function startTimer() {
  if (timer) clearInterval(timer)
  if (remaining.value > 0) {
    timer = setInterval(() => {
      remaining.value--
      if (remaining.value <= 0) {
        remaining.value = 0
        if (timer) clearInterval(timer)
      }
    }, 1000)
  }
}

async function sendHug() {
  if (loading.value || remaining.value > 0) return
  loading.value = true
  try {
    await hugsStore.sendHug(props.userId)
    animating.value = true
    showEmoji.value = true
    setTimeout(() => {
      animating.value = false
    }, 600)
    setTimeout(() => {
      showEmoji.value = false
    }, 1000)
    emit('hugged')
    await loadCooldown()
  } catch (e: any) {
    const msg = e.response?.data?.message || 'Ошибка'
    alert(msg)
  } finally {
    loading.value = false
  }
}

function formatTime(seconds: number): string {
  const m = Math.floor(seconds / 60)
  const s = seconds % 60
  return `${m}:${s.toString().padStart(2, '0')}`
}

onMounted(loadCooldown)
onUnmounted(() => {
  if (timer) clearInterval(timer)
})
</script>

<template>
  <div class="relative inline-block">
    <button
      @click="sendHug"
      :disabled="loading || remaining > 0"
      :class="[
        'relative overflow-hidden transition-all duration-300',
        compact
          ? 'px-4 py-2 rounded-lg text-sm'
          : 'px-8 py-4 rounded-2xl text-lg font-bold',
        remaining > 0
          ? 'bg-indigo-800/50 text-indigo-400 cursor-not-allowed'
          : 'bg-gradient-to-r from-pink-500 to-purple-500 hover:from-pink-400 hover:to-purple-400 text-white shadow-lg hover:shadow-pink-500/25 hover:scale-105 active:scale-95',
        animating ? 'hug-animate' : '',
      ]"
    >
      <span v-if="loading">⏳</span>
      <span v-else-if="remaining > 0">⏱️ {{ formatTime(remaining) }}</span>
      <span v-else>🤗 {{ compact ? 'Обнять' : 'Обнять!' }}</span>
    </button>

    <Transition name="fade">
      <span
        v-if="showEmoji"
        class="absolute -top-2 left-1/2 -translate-x-1/2 text-3xl float-emoji pointer-events-none"
      >
        🤗
      </span>
    </Transition>
  </div>
</template>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
