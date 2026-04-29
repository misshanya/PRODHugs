<script setup lang="ts">
import { useAuthStore } from '@/stores/auth'
import { useOnlineStore } from '@/stores/online'
import { Avatar, AvatarFallback } from '@/components/ui/avatar'
import HugButton from './HugButton.vue'

const props = defineProps<{
  user: {
    id: string
    username: string
    role: string
    display_name?: string | null
  }
}>()

const auth = useAuthStore()
const onlineStore = useOnlineStore()
const isMe = auth.user?.id === props.user.id
</script>

<template>
  <div
    class="flex items-center justify-between rounded-[10px] border p-3 transition-colors hover:bg-[#002D20]"
  >
    <RouterLink :to="`/user/${user.id}`" class="flex items-center gap-3 flex-1 min-w-0">
      <div class="relative shrink-0">
        <Avatar class="size-9">
          <AvatarFallback class="text-xs">
            {{ (user.display_name || user.username).slice(0, 2).toUpperCase() }}
          </AvatarFallback>
        </Avatar>
        <span
          v-if="onlineStore.isOnline(user.id)"
          class="absolute -bottom-0.5 -right-0.5 flex size-3 items-center justify-center rounded-full border-2 border-background bg-emerald-500"
        />
      </div>
      <div class="min-w-0">
        <p class="text-sm font-medium truncate">
          {{ user.display_name || user.username }}
        </p>
        <p class="text-xs text-muted-foreground mt-1">
          <template v-if="user.display_name">@{{ user.username }} · </template>
          {{ user.role === 'admin' ? 'Админ' : 'Пользователь' }}
        </p>
      </div>
    </RouterLink>
    <HugButton v-if="!isMe" :userId="user.id" :username="user.display_name || user.username" size="sm" />
  </div>
</template>
