<script setup lang="ts">
import { computed } from 'vue'
import type { StreakCalendarDay } from '@/stores/hugs'
import type { StreakInfo } from '@/stores/hugs'
import { getStreakTier } from '@/lib/utils'
import { Flame } from 'lucide-vue-next'
import StreakBadge from '@/components/StreakBadge.vue'
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip'

const props = defineProps<{
  streak: StreakInfo
  calendar: StreakCalendarDay[]
  /** Whether current user is user_a in the pair (for today's progress display) */
  viewerIsA: boolean
}>()

const tier = computed(() => getStreakTier(props.streak.tier_key))

// Build a 90-day grid (last ~3 months)
const calendarGrid = computed(() => {
  const days: Array<{
    date: string
    dayOfMonth: number
    month: number
    completed: boolean
    partial: boolean
    hugCount: number
  }> = []

  const today = new Date()
  const calendarMap = new Map<string, StreakCalendarDay>()
  for (const day of props.calendar) {
    calendarMap.set(day.date, day)
  }

  for (let i = 89; i >= 0; i--) {
    const d = new Date(today)
    d.setDate(d.getDate() - i)
    const dateStr = d.toISOString().split('T')[0]!
    const entry = calendarMap.get(dateStr)

    days.push({
      date: dateStr,
      dayOfMonth: d.getDate(),
      month: d.getMonth(),
      completed: entry?.completed ?? false,
      partial: entry ? !entry.completed && entry.hug_count > 0 : false,
      hugCount: entry?.hug_count ?? 0,
    })
  }

  return days
})

// Today's progress
const viewerHugged = computed(() =>
  props.viewerIsA ? props.streak.a_hugged_today : props.streak.b_hugged_today,
)
const partnerHugged = computed(() =>
  props.viewerIsA ? props.streak.b_hugged_today : props.streak.a_hugged_today,
)

function cellColor(day: { completed: boolean; partial: boolean }) {
  if (day.completed) {
    return tier.value ? `${tier.value.bgClass} ${tier.value.borderClass} border` : 'bg-emerald-500/30 border border-emerald-500/50'
  }
  if (day.partial) {
    return 'bg-muted-foreground/20 border border-muted-foreground/30'
  }
  return 'bg-muted/50'
}
</script>

<template>
  <div class="space-y-3">
    <!-- Header: streak count + badge -->
    <div class="flex items-center gap-2">
      <Flame class="size-5" :class="tier?.textClass ?? 'text-muted-foreground'" />
      <span class="text-lg font-semibold tabular-nums">{{ streak.current_streak }}</span>
      <span class="text-sm text-muted-foreground">
        {{ streak.current_streak === 1 ? 'день' : streak.current_streak < 5 ? 'дня' : 'дней' }}
      </span>
      <StreakBadge
        v-if="streak.tier_key"
        :tier-key="streak.tier_key"
        :tier-name="streak.tier_name"
      />
    </div>

    <!-- Best streak -->
    <p v-if="streak.best_streak > streak.current_streak" class="text-xs text-muted-foreground">
      Лучшая серия: {{ streak.best_streak }} дн.
    </p>

    <!-- Today's progress -->
    <div class="flex items-center gap-3 text-xs">
      <span :class="viewerHugged ? 'text-emerald-400' : 'text-muted-foreground'">
        {{ viewerHugged ? 'Вы обняли' : 'Вы ещё не обняли' }}
      </span>
      <span class="text-muted-foreground">/</span>
      <span :class="partnerHugged ? 'text-emerald-400' : 'text-muted-foreground'">
        {{ partnerHugged ? 'Вас обняли' : 'Вас ещё не обняли' }}
      </span>
    </div>

    <!-- Calendar grid -->
    <TooltipProvider :delay-duration="100">
      <div class="grid grid-cols-[repeat(13,1fr)] gap-0.5">
        <Tooltip v-for="day in calendarGrid" :key="day.date">
          <TooltipTrigger as-child>
            <div
              class="aspect-square w-full rounded-[3px]"
              :class="cellColor(day)"
            />
          </TooltipTrigger>
          <TooltipContent side="top" class="text-xs">
            <p>{{ day.date }}</p>
            <p v-if="day.hugCount > 0">
              {{ day.hugCount }} {{ day.hugCount === 1 ? 'обнимашка' : 'обнимашек' }}
              {{ day.completed ? '(взаимно)' : '(не взаимно)' }}
            </p>
            <p v-else class="text-muted-foreground">Нет обнимашек</p>
          </TooltipContent>
        </Tooltip>
      </div>
    </TooltipProvider>

    <!-- Next tier info -->
    <p v-if="streak.next_tier_at" class="text-xs text-muted-foreground">
      До следующей лиги: {{ streak.next_tier_at - streak.current_streak }} дн.
    </p>
  </div>
</template>
