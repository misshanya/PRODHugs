<script setup lang="ts">
import { computed, ref } from 'vue'
import { Badge } from '@/components/ui/badge'
import { getStreakTier } from '@/lib/utils'
import { Flame } from 'lucide-vue-next'
import StreakLeagueModal from '@/components/StreakLeagueModal.vue'

const props = defineProps<{
  tierKey: string
  tierName?: string
  streakDays?: number
}>()

const tier = computed(() => getStreakTier(props.tierKey))
const badgeClasses = computed(() => tier.value?.badgeClasses ?? '')
const modalOpen = ref(false)

function onClick(e: Event) {
  e.stopPropagation()
  e.preventDefault()
  modalOpen.value = true
}
</script>

<template>
  <span v-if="tier" class="inline-flex">
    <Badge
      variant="outline"
      :class="[badgeClasses, 'cursor-pointer']"
      @click="onClick"
    >
      <Flame class="mr-0.5 size-3" />
      <span v-if="streakDays" class="mr-0.5 tabular-nums">{{ streakDays }}</span>
      {{ tierName || tier.name }}
    </Badge>
    <StreakLeagueModal v-model:open="modalOpen" />
  </span>
</template>
