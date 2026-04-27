<script setup lang="ts">
import { computed } from 'vue'
import { Badge } from '@/components/ui/badge'

const props = defineProps<{
  rank: string
}>()

const rankStyles: Record<string, string> = {
  Новичок: 'bg-[#b2c6c0]/15 text-[#b2c6c0] border-[#b2c6c0]/20',
  Обнимашка: 'bg-teal-400/15 text-teal-300 border-teal-400/20',
  Дружелюбный: 'bg-emerald-400/15 text-emerald-300 border-emerald-400/20',
  'Мастер обнимашек': 'bg-cyan-400/15 text-cyan-300 border-cyan-400/20',
  Легенда: 'bg-prod-yellow/15 text-prod-yellow border-prod-yellow/20',
  'Бог обнимашек': 'bg-amber-400/15 text-amber-300 border-amber-400/20',
}

const style = computed(() => rankStyles[props.rank] ?? rankStyles['Новичок']!)
</script>

<template>
  <Badge variant="outline" :class="style">
    {{ rank }}
  </Badge>
</template>
