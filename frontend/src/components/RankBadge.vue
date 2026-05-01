<script setup lang="ts">
import { computed } from 'vue'
import { Badge } from '@/components/ui/badge'

const props = defineProps<{
  rank: string
}>()

// Map rank names (including gendered forms) to styles.
// Uses startsWith matching to handle gender suffixes like "Нетактильный(ая)".
const rankPatterns: Array<{ pattern: string; style: string }> = [
  { pattern: 'Милашка', style: 'bg-amber-400/15 text-amber-300 border-amber-400/20' },
  { pattern: 'Легенда', style: 'bg-prod-yellow/15 text-prod-yellow border-prod-yellow/20' },
  { pattern: 'Обнимастер', style: 'bg-cyan-400/15 text-cyan-300 border-cyan-400/20' },
  { pattern: 'Тактильн', style: 'bg-emerald-400/15 text-emerald-300 border-emerald-400/20' },
  { pattern: 'Неопытн', style: 'bg-teal-400/15 text-teal-300 border-teal-400/20' },
  { pattern: 'Нетактильн', style: 'bg-[#b2c6c0]/15 text-[#b2c6c0] border-[#b2c6c0]/20' },
]

const defaultStyle = 'bg-[#b2c6c0]/15 text-[#b2c6c0] border-[#b2c6c0]/20'

const style = computed(() => {
  const match = rankPatterns.find((p) => props.rank.startsWith(p.pattern))
  return match?.style ?? defaultStyle
})
</script>

<template>
  <Badge variant="outline" :class="style">
    {{ rank }}
  </Badge>
</template>
