<script setup lang="ts">
import { ref, computed } from 'vue'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { usersApi } from '@/api/client'
import { useAuthStore } from '@/stores/auth'
import { toast } from 'vue-sonner'
import { Loader2, Star } from 'lucide-vue-next'

const props = defineProps<{
  open: boolean
}>()

const emit = defineEmits<{
  (e: 'update:open', value: boolean): void
  (e: 'success'): void
}>()

const auth = useAuthStore()
const loading = ref(false)

const durationHours = ref(24)
const message = ref('')

const PROMOTION_COST_PER_HOUR = 5
const cost = computed(() => durationHours.value * PROMOTION_COST_PER_HOUR)

const canAfford = computed(() => {
  const balance = auth.user?.balance ?? 0
  return balance >= cost.value
})

async function handlePromote() {
  if (!canAfford.value) return
  loading.value = true
  try {
    await usersApi.promote(durationHours.value, message.value || undefined)
    toast.success('Успешно!', {
      description: `Вы в топе на ${durationHours.value}ч.`,
    })
    emit('success')
    emit('update:open', false)
    // Update local user state if needed (or just refresh)
    await auth.fetchMe()
  } catch (error: any) {
    toast.error('Ошибка', {
      description: error.response?.data?.message || 'Не удалось активировать продвижение',
    })
  } finally {
    loading.value = false
  }
}

const durations = [
  { label: '1ч', value: 1 },
  { label: '6ч', value: 6 },
  { label: '24ч', value: 24 },
  { label: '2д', value: 48 },
  { label: '1 нед', value: 168 },
]
</script>

<template>
  <Dialog :open="open" @update:open="emit('update:open', $event)">
    <DialogContent class="sm:max-w-[425px]">
      <DialogHeader>
        <DialogTitle class="flex items-center gap-2">
          <Star class="size-5 text-prod-yellow fill-prod-yellow" />
          Продвижение профиля
        </DialogTitle>
        <DialogDescription>
          Вас увидят первым в списке пользователей!
        </DialogDescription>
      </DialogHeader>

      <div class="grid gap-4 py-4">
        <div class="space-y-2">
          <Label>Длительность</Label>
          <div class="flex flex-wrap gap-2">
            <Button
              v-for="d in durations"
              :key="d.value"
              type="button"
              variant="outline"
              size="sm"
              :class="{ 'border-prod-yellow bg-prod-yellow/10': durationHours === d.value }"
              @click="durationHours = d.value"
            >
              {{ d.label }}
            </Button>
          </div>
        </div>

        <div class="space-y-2">
          <Label for="message">Сообщение (необязательно)</Label>
          <Input
            id="message"
            v-model="message"
            placeholder="Например: Спонсорское место"
            maxlength="100"
          />
        </div>

        <div class="rounded-lg bg-muted p-3">
          <div class="flex items-center justify-between text-sm">
            <span>Стоимость:</span>
            <span class="font-bold text-prod-yellow">{{ cost }} монет</span>
          </div>
          <div class="flex items-center justify-between text-sm mt-1">
            <span>Ваш баланс:</span>
            <span>{{ auth.user?.balance ?? 0 }} монет</span>
          </div>
        </div>
        
        <p v-if="!canAfford" class="text-xs text-destructive text-center">
          Недостаточно монет для этой длительности
        </p>
      </div>

      <DialogFooter>
        <Button
          class="w-full bg-prod-yellow text-black hover:bg-prod-yellow/90"
          :disabled="loading || !canAfford"
          @click="handlePromote"
        >
          <Loader2 v-if="loading" class="mr-2 size-4 animate-spin" />
          Подняться в топ
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
