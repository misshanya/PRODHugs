<script setup lang="ts">
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { streakTiers } from '@/lib/utils'
import { Flame } from 'lucide-vue-next'

defineProps<{
  open: boolean
}>()

defineEmits<{
  'update:open': [value: boolean]
}>()
</script>

<template>
  <Dialog :open="open" @update:open="$emit('update:open', $event)">
    <DialogContent class="sm:max-w-md">
      <DialogHeader>
        <DialogTitle class="flex items-center gap-2">
          <Flame class="size-5 text-amber-400" />
          Лиги обнимашек
        </DialogTitle>
        <DialogDescription>
          Обнимайте друг друга каждый день, чтобы повышать лигу. Оба должны отправить минимум 1
          обнимашку в день, чтобы серия продолжилась.
        </DialogDescription>
      </DialogHeader>

      <div class="mt-4 space-y-2">
        <div
          v-for="tier in streakTiers"
          :key="tier.key"
          class="flex items-center justify-between rounded-lg border px-3 py-2.5"
          :class="tier.bgClass"
        >
          <div class="flex items-center gap-2">
            <Flame class="size-4" :class="tier.textClass" />
            <span class="text-sm font-medium" :class="tier.textClass">{{ tier.name }}</span>
          </div>
          <span class="text-xs text-muted-foreground">{{ tier.minDays }}+ дней</span>
        </div>
      </div>

      <div class="mt-4 space-y-2 text-xs text-muted-foreground">
        <p>
          Серия считается по UTC-дням. Если хотя бы один из вас не отправит обнимашку за день —
          серия обнуляется.
        </p>
        <p>
          Лига отображается на обнимашках в ленте и сохраняется навсегда на момент принятия.
        </p>
      </div>
    </DialogContent>
  </Dialog>
</template>
