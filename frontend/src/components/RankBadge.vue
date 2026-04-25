<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  rank: string
  size?: 'sm' | 'md' | 'lg'
}>()

const rankConfig: Record<string, { emoji: string; color: string }> = {
  Новичок: { emoji: '🌱', color: 'text-green-400 bg-green-900/30 border-green-700/30' },
  Обнимашка: { emoji: '🤗', color: 'text-blue-400 bg-blue-900/30 border-blue-700/30' },
  Дружелюбный: { emoji: '💛', color: 'text-yellow-400 bg-yellow-900/30 border-yellow-700/30' },
  'Мастер объятий': { emoji: '⭐', color: 'text-purple-400 bg-purple-900/30 border-purple-700/30' },
  Легенда: { emoji: '👑', color: 'text-amber-400 bg-amber-900/30 border-amber-700/30' },
  'Бог объятий': { emoji: '🔥', color: 'text-red-400 bg-red-900/30 border-red-700/30' },
}

const config = computed(() => rankConfig[props.rank] ?? rankConfig['Новичок']!)
const sizeClass = computed(() => {
  switch (props.size) {
    case 'sm':
      return 'text-xs px-2 py-0.5'
    case 'lg':
      return 'text-base px-4 py-2'
    default:
      return 'text-sm px-3 py-1'
  }
})
</script>

<template>
  <span
    :class="[
      'inline-flex items-center gap-1 rounded-full border font-medium',
      config.color,
      sizeClass,
    ]"
  >
    <span>{{ config.emoji }}</span>
    <span>{{ rank }}</span>
  </span>
</template>
