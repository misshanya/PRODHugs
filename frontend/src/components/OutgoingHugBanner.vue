<script setup lang="ts">
import { ref, computed } from 'vue'
import { Heart, X } from 'lucide-vue-next'
import { toast } from 'vue-sonner'
import { useHugsStore } from '@/stores/hugs'
import { useAuthStore } from '@/stores/auth'
import { suggestVerb } from '@/lib/utils'
import { Button } from '@/components/ui/button'

const hugsStore = useHugsStore()
const auth = useAuthStore()
const cancelling = ref(false)

const outgoing = computed(() => hugsStore.outgoingHug)

function relativeTime(dateStr: string): string {
  const diff = Math.floor((Date.now() - new Date(dateStr).getTime()) / 1000)
  if (diff < 60) return 'только что'
  if (diff < 3600) return `${Math.floor(diff / 60)} мин. назад`
  if (diff < 86400) return `${Math.floor(diff / 3600)} ч. назад`
  return new Date(dateStr).toLocaleDateString('ru-RU', {
    day: 'numeric',
    month: 'short',
  })
}

async function cancel() {
  if (!outgoing.value || cancelling.value) return
  cancelling.value = true
  try {
    await hugsStore.cancelOutgoing(outgoing.value.id)
    toast('Предложение отменено')
  } catch (e: unknown) {
    const err = e as { response?: { data?: { message?: string } } }
    toast.error(err.response?.data?.message || 'Не удалось отменить')
  } finally {
    cancelling.value = false
  }
}
</script>

<template>
  <Transition name="banner">
    <div
      v-if="outgoing"
      class="flex items-center gap-3 rounded-lg border border-prod-yellow/30 bg-prod-yellow/5 p-3"
    >
      <Heart class="size-4 shrink-0 text-prod-yellow outgoing-pulse" />
      <div class="min-w-0 flex-1 text-sm">
        <span class="text-muted-foreground"
          >Ты {{ suggestVerb(auth.user?.gender) }} обняться
        </span>
        <RouterLink :to="`/user/${outgoing.receiver_id}`" class="font-medium hover:underline">
          {{ outgoing.receiver_username }}
        </RouterLink>
        <span class="ml-1.5 text-[10px] text-muted-foreground">
          {{ relativeTime(outgoing.created_at) }}
        </span>
      </div>
      <Button
        variant="ghost"
        size="sm"
        class="shrink-0 gap-1 rounded-[21px]"
        :disabled="cancelling"
        @click="cancel"
      >
        <X class="size-3.5" />
        Отменить
      </Button>
    </div>
  </Transition>
</template>

<style scoped>
.outgoing-pulse {
  animation: outgoing-pulse-anim 2s ease-in-out infinite;
}

@keyframes outgoing-pulse-anim {
  0%,
  100% {
    opacity: 1;
  }
  50% {
    opacity: 0.5;
  }
}

.banner-enter-active {
  transition: all 0.4s cubic-bezier(0.22, 1, 0.36, 1);
}

.banner-leave-active {
  transition: all 0.3s ease-out;
}

.banner-enter-from {
  opacity: 0;
  transform: translateY(-8px);
}

.banner-leave-to {
  opacity: 0;
  transform: translateY(-8px);
}
</style>
