<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, watch } from 'vue'
import { Heart, Clock, Loader2, Hourglass, ChevronDown } from 'lucide-vue-next'
import { toast } from 'vue-sonner'
import { useAuthStore } from '@/stores/auth'
import { useHugsStore, type CooldownInfo, type IntimacyInfo, type HugType } from '@/stores/hugs'
import { suggestVerb, hugTypeLabel } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'

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
const intimacy = ref<IntimacyInfo | null>(null)
let timer: ReturnType<typeof setInterval> | null = null

const availableHugTypes = computed<HugType[]>(() => {
  if (!intimacy.value) return ['standard']
  return intimacy.value.available_hug_types
})
const hasMultipleTypes = computed(() => availableHugTypes.value.length > 1)

// Pending state computeds
const allSlotsFull = computed(
  () => hugsStore.outgoingHugs.length >= hugsStore.slotInfo.total_slots,
)
const isPendingWithThisUser = computed(
  () => hugsStore.outgoingHugs.some((h) => h.receiver_id === props.userId),
)
const hasIncomingPending = computed(() => hugsStore.inbox.some((h) => h.giver_id === props.userId))

const isDisabled = computed(
  () =>
    loading.value ||
    remaining.value > 0 ||
    allSlotsFull.value ||
    hasIncomingPending.value ||
    isPendingWithThisUser.value,
)

const buttonVariant = computed(() => {
  if (
    remaining.value > 0 ||
    allSlotsFull.value ||
    hasIncomingPending.value ||
    isPendingWithThisUser.value
  )
    return 'secondary'
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
  try {
    intimacy.value = await hugsStore.getPairIntimacy(props.userId)
  } catch {
    // Ignore — defaults to standard only
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

async function suggest(hugType?: string) {
  if (suggesting || isDisabled.value) return
  suggesting = true
  loading.value = true
  try {
    await hugsStore.suggestHug(props.userId, hugType)
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
    <!-- Single button when only standard type or disabled state -->
    <Button
      v-if="!hasMultipleTypes || isDisabled"
      @click="suggest()"
      :disabled="isDisabled"
      :size="size ?? 'default'"
      :variant="buttonVariant"
      class="rounded-[21px]"
    >
      <Loader2 v-if="loading" class="size-4 animate-spin" />
      <Clock v-else-if="remaining > 0" class="size-4" />
      <Hourglass v-else-if="allSlotsFull || hasIncomingPending" class="size-4" />
      <Heart v-else class="size-4" />
      <span v-if="remaining > 0">{{ formatTime(remaining) }}</span>
      <span v-else-if="isPendingWithThisUser">Ожидание...</span>
      <span v-else-if="hasIncomingPending">Ждет твоего ответа</span>
      <span v-else-if="allSlotsFull">Все слоты заняты</span>
      <span v-else>Обняться</span>
    </Button>

    <!-- Split button with dropdown when multiple types are unlocked -->
    <div v-else class="inline-flex">
      <Button
        @click="suggest()"
        :disabled="isDisabled"
        :size="size ?? 'default'"
        :variant="buttonVariant"
        class="rounded-l-[21px] rounded-r-none"
      >
        <Loader2 v-if="loading" class="size-4 animate-spin" />
        <Heart v-else class="size-4" />
        <span>Обняться</span>
      </Button>
      <DropdownMenu>
        <DropdownMenuTrigger as-child>
          <Button
            :disabled="isDisabled"
            :size="size ?? 'default'"
            :variant="buttonVariant"
            class="rounded-l-none rounded-r-[21px] border-l border-background/20 px-1.5"
          >
            <ChevronDown class="size-3.5" />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end" class="w-40">
          <DropdownMenuItem
            v-for="ht in availableHugTypes"
            :key="ht"
            @click="suggest(ht)"
          >
            {{ hugTypeLabel(ht) }}
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
  </div>
</template>
