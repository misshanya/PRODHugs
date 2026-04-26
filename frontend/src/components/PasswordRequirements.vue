<script setup lang="ts">
import { computed } from 'vue'
import { Progress } from '@/components/ui/progress'

const props = defineProps<{
  password: string
}>()

const PASSWORD_MIN = 8
const PASSWORD_MAX = 128
const HAS_LETTER = /[a-zA-Z]/
const HAS_DIGIT = /[0-9]/
const HAS_SPECIAL = /[^a-zA-Z0-9\s]/

const meetsLength = computed(() => {
  const len = props.password.length
  return len >= PASSWORD_MIN && len <= PASSWORD_MAX
})

const hasLetter = computed(() => HAS_LETTER.test(props.password))
const hasDigit = computed(() => HAS_DIGIT.test(props.password))
const hasSpecial = computed(() => HAS_SPECIAL.test(props.password))

const requirements = computed(() => [
  { id: 'length', label: `Минимум ${PASSWORD_MIN} символов`, met: meetsLength.value },
  { id: 'letter', label: 'Хотя бы одна латинская буква', met: hasLetter.value },
  { id: 'digit', label: 'Хотя бы одна цифра', met: hasDigit.value },
  { id: 'special', label: 'Хотя бы один спецсимвол', met: hasSpecial.value },
])

const metCount = computed(() => requirements.value.filter((r) => r.met).length)
const progressValue = computed(() => (metCount.value / requirements.value.length) * 100)

</script>

<template>
  <div class="mt-2 space-y-2 rounded-md border border-border bg-card p-3">
    <Progress :model-value="progressValue" class="h-1.5" />
    <ul class="space-y-1.5">
      <li
        v-for="req in requirements"
        :key="req.id"
        class="flex items-center gap-2 text-xs"
        :class="req.met ? 'text-primary' : 'text-muted-foreground'"
      >
        <svg
          v-if="req.met"
          xmlns="http://www.w3.org/2000/svg"
          width="14"
          height="14"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="3"
          stroke-linecap="round"
          stroke-linejoin="round"
          class="text-primary"
        >
          <polyline points="20 6 9 17 4 12" />
        </svg>
        <svg
          v-else
          xmlns="http://www.w3.org/2000/svg"
          width="14"
          height="14"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="3"
          stroke-linecap="round"
          stroke-linejoin="round"
          class="opacity-50"
        >
          <line x1="18" y1="6" x2="6" y2="18" />
          <line x1="6" y1="6" x2="18" y2="18" />
        </svg>
        <span>{{ req.label }}</span>
      </li>
    </ul>
  </div>
</template>
