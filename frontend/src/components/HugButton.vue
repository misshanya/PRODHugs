<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { Heart, Clock, Loader2 } from 'lucide-vue-next'
import { toast } from 'vue-sonner'
import { useHugsStore, type CooldownInfo } from '@/stores/hugs'
import { Button } from '@/components/ui/button'
import HugExplosion from '@/components/HugExplosion.vue'

const props = defineProps<{
  userId: string
  size?: 'default' | 'sm' | 'lg'
}>()

const emit = defineEmits<{
  hugged: []
}>()

const hugsStore = useHugsStore()
const loading = ref(false)
const cooldown = ref<CooldownInfo | null>(null)
const remaining = ref(0)
const animating = ref(false)
const showExplosion = ref(false)
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
    showExplosion.value = true
    setTimeout(() => {
      animating.value = false
    }, 800)
    toast.success('Объятие отправлено!')
    emit('hugged')
    await loadCooldown()
  } catch (e: any) {
    toast.error(e.response?.data?.message || 'Не удалось отправить объятие')
  } finally {
    loading.value = false
  }
}

function onExplosionDone() {
  showExplosion.value = false
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
  <div class="relative inline-flex">
    <Button
      @click="sendHug"
      :disabled="loading || remaining > 0"
      :size="size ?? 'default'"
      :variant="remaining > 0 ? 'secondary' : 'default'"
      :class="[animating ? 'hug-animate' : '']"
    >
      <Loader2 v-if="loading" class="size-4 animate-spin" />
      <Clock v-else-if="remaining > 0" class="size-4" />
      <Heart v-else class="size-4" />
      <span v-if="remaining > 0">{{ formatTime(remaining) }}</span>
      <span v-else>Обнять</span>
    </Button>
    <HugExplosion v-if="showExplosion" @done="onExplosionDone" />
  </div>
</template>
