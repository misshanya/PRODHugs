<script setup lang="ts">
import HugButton from './HugButton.vue'
import { useAuthStore } from '@/stores/auth'

const props = defineProps<{
  user: {
    id: string
    username: string
    role: string
  }
}>()

const auth = useAuthStore()
const isMe = auth.user?.id === props.user.id
</script>

<template>
  <div class="card flex items-center justify-between hover:border-primary/50 transition-all group">
    <RouterLink :to="`/user/${user.id}`" class="flex items-center gap-4 flex-1 min-w-0">
      <div
        class="w-12 h-12 rounded-full bg-gradient-to-br from-primary to-accent flex items-center justify-center text-lg font-bold shrink-0"
      >
        {{ user.username[0]?.toUpperCase() }}
      </div>
      <div class="min-w-0">
        <p class="font-semibold text-white group-hover:text-primary-light transition-colors truncate">
          {{ user.username }}
        </p>
        <p class="text-xs text-indigo-400">{{ user.role === 'admin' ? 'Админ' : 'Пользователь' }}</p>
      </div>
    </RouterLink>

    <HugButton v-if="!isMe" :userId="user.id" compact />
  </div>
</template>
