<script setup lang="ts">
import { computed } from 'vue'
import { Badge } from '@/components/ui/badge'

const props = defineProps<{
  rank: string
}>()

const rankStyles: Record<string, string> = {
  'Новичок': 'bg-zinc-500/15 text-zinc-400 border-zinc-500/20',
  'Обнимашка': 'bg-blue-500/15 text-blue-400 border-blue-500/20',
  'Дружелюбный': 'bg-green-500/15 text-green-400 border-green-500/20',
  'Мастер обнимашек': 'bg-purple-500/15 text-purple-400 border-purple-500/20',
  'Легенда': 'bg-amber-500/15 text-amber-400 border-amber-500/20',
  'Бог обнимашек': 'bg-red-500/15 text-red-400 border-red-500/20',
}

const style = computed(() => rankStyles[props.rank] ?? rankStyles['Новичок']!)
</script>

<template>
  <Badge variant="outline" :class="style">
    {{ rank }}
  </Badge>
</template>
