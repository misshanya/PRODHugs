<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { Coins, LogOut, Settings } from 'lucide-vue-next'
import { useAuthStore } from '@/stores/auth'
import { useHugsStore } from '@/stores/hugs'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Avatar, AvatarFallback } from '@/components/ui/avatar'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import SettingsDialog from '@/components/SettingsDialog.vue'

const auth = useAuthStore()
const hugs = useHugsStore()
const settingsOpen = ref(false)

onMounted(() => {
  hugs.fetchBalance()
})
</script>

<template>
  <div class="flex flex-1 items-center justify-between">
    <!-- Mobile-only branding (replaces sidebar trigger on small screens) -->
    <div class="flex items-center gap-2 md:hidden">
      <img src="/logo.webp" alt="PROD" class="size-8 shrink-0 rounded-lg object-contain" />
      <span class="text-sm font-semibold text-foreground"><span class="font-bold">PROD</span>нимашки</span>
    </div>
    <div class="hidden md:block" />
    <div class="flex items-center gap-3">
      <Badge variant="secondary" class="gap-1.5 font-mono tabular-nums bg-prod-yellow/15 text-prod-yellow border-prod-yellow/20">
        <Coins class="size-3.5" />
        {{ hugs.balance?.amount ?? 0 }}
      </Badge>

      <DropdownMenu>
        <DropdownMenuTrigger as-child>
          <Button variant="ghost" size="icon" class="rounded-full">
            <Avatar class="size-8">
              <AvatarFallback class="text-xs">
                {{ auth.user?.username?.slice(0, 2)?.toUpperCase() }}
              </AvatarFallback>
            </Avatar>
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end" class="w-48">
          <DropdownMenuLabel>{{ auth.user?.username }}</DropdownMenuLabel>
          <DropdownMenuSeparator />
          <DropdownMenuItem @click="settingsOpen = true">
            <Settings class="size-4" />
            Настройки
          </DropdownMenuItem>
          <DropdownMenuSeparator />
          <DropdownMenuItem @click="auth.logout()" class="text-destructive">
            <LogOut class="size-4" />
            Выйти
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>

      <SettingsDialog v-model:open="settingsOpen" />
    </div>
  </div>
</template>
