<script setup lang="ts">
import { useAuthStore } from '@/stores/auth'
import { Avatar, AvatarFallback } from '@/components/ui/avatar'
import HugButton from './HugButton.vue'

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
  <div class="flex items-center justify-between rounded-lg border p-3 transition-colors hover:bg-accent/50">
    <RouterLink :to="`/user/${user.id}`" class="flex items-center gap-3 flex-1 min-w-0">
      <Avatar class="size-9">
        <AvatarFallback class="text-xs">
          {{ user.username.slice(0, 2).toUpperCase() }}
        </AvatarFallback>
      </Avatar>
      <div class="min-w-0">
        <p class="text-sm font-medium leading-none truncate">{{ user.username }}</p>
        <p class="text-xs text-muted-foreground mt-1">
          {{ user.role === 'admin' ? 'Админ' : 'Пользователь' }}
        </p>
      </div>
    </RouterLink>
    <HugButton v-if="!isMe" :userId="user.id" :username="user.username" size="sm" />
  </div>
</template>
