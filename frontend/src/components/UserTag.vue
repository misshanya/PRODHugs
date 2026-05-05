<script setup lang="ts">
import { Badge } from '@/components/ui/badge'

defineProps<{
  tag?: string | null
  size?: 'sm' | 'md'
}>()
</script>

<template>
  <Badge
    v-if="tag"
    variant="outline"
    :class="size === 'md' ? 'text-[10px] px-1.5 py-0' : 'text-[9px] px-1.5 py-0'"
    class="shrink-0"
  >
    {{ tag }}
  </Badge>
</template>
