<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, watch } from 'vue'
import { Heart, Clock, Loader2, Hourglass } from 'lucide-vue-next'
import { toast } from 'vue-sonner'
import { useAuthStore } from '@/stores/auth'
import { useHugsStore, type CooldownInfo } from '@/stores/hugs'
import { suggestVerb } from '@/lib/utils'
import { Button } from '@/components/ui/button'

const props = defineProps<{
  userId: string
  username: string
  size?: 'default' | 'sm' | 'lg'
}>()

const emit = defineEmits<{
  hugged: []
}>()

const auth = useAuthStore()
const hugsStore = useHugsStore()
const loading = ref(false)
const cooldown = ref<CooldownInfo | null>(null)
const remaining = ref(0)
const btnRef = ref<HTMLButtonElement | null>(null)
let timer: ReturnType<typeof setInterval> | null = null

// Pending state computeds
const hasGlobalPending = computed(() => !!hugsStore.outgoingHug)
const isPendingWithThisUser = computed(() => hugsStore.outgoingHug?.receiver_id === props.userId)
const hasIncomingPending = computed(() => hugsStore.inbox.some(h => h.giver_id === props.userId))

const isDisabled = computed(() => loading.value || remaining.value > 0 || hasGlobalPending.value || hasIncomingPending.value)

const buttonVariant = computed(() => {
  if (remaining.value > 0 || hasGlobalPending.value || hasIncomingPending.value) return 'secondary'
  return 'yellow'
})

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

let suggesting = false

async function suggest() {
  if (suggesting || loading.value || remaining.value > 0 || hasGlobalPending.value) return
  suggesting = true
  loading.value = true
  try {
    await hugsStore.suggestHug(props.userId)
    toast.success(`Ты ${suggestVerb(auth.user?.gender)} обнимашку ${props.username}!`)
    emit('hugged')
  } catch (e: unknown) {
    const err = e as { response?: { data?: { message?: string } } }
    toast.error(err.response?.data?.message || `Не удалось предложить обнимашку ${props.username}`)
  } finally {
    loading.value = false
    suggesting = false
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

watch(() => hugsStore.cooldownRefreshes[props.userId], (newVal, oldVal) => {
  if (newVal && newVal !== oldVal) {
    loadCooldown()
  }
})
</script>

<template>
  <div ref="btnRef" class="relative inline-flex">
    <Button
      @click="suggest"
      :disabled="isDisabled"
      :size="size ?? 'default'"
      :variant="buttonVariant"
      class="rounded-[21px]"
    >
      <Loader2 v-if="loading" class="size-4 animate-spin" />
      <Clock v-else-if="remaining > 0" class="size-4" />
      <Hourglass v-else-if="hasGlobalPending || hasIncomingPending" class="size-4" />
      <Heart v-else class="size-4" />
      <span v-if="remaining > 0">{{ formatTime(remaining) }}</span>
      <span v-else-if="isPendingWithThisUser">Ожидание...</span>
      <span v-else-if="hasIncomingPending">Ждет твоего ответа</span>
      <span v-else-if="hasGlobalPending">Ты уже ждешь ответа от другого</span>
      <span v-else>Предложить обняться</span>
    </Button>
  </div>
</template>
