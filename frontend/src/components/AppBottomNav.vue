<script setup lang="ts">
import { useRoute } from 'vue-router'
import { computed } from 'vue'
import { LayoutDashboard, Users, Newspaper, Trophy, Shield, Heart } from 'lucide-vue-next'
import { useAuthStore } from '@/stores/auth'
import { useHugsStore } from '@/stores/hugs'

const route = useRoute()
const auth = useAuthStore()
const hugsStore = useHugsStore()
const currentPath = computed(() => route.path)
const inboxCount = computed(() => hugsStore.inboxCount)

const baseItems = [
  { title: 'Главная', url: '/dashboard', icon: LayoutDashboard },
  { title: 'Люди', url: '/users', icon: Users },
  { title: 'Лента', url: '/feed', icon: Newspaper },
  { title: 'Связи', url: '/connections', icon: Heart },
  { title: 'Рейтинг', url: '/leaderboard', icon: Trophy },
]

const items = computed(() => {
  if (auth.user?.role === 'admin') {
    return [...baseItems, { title: 'Админ', url: '/admin', icon: Shield }]
  }
  return baseItems
})

function isActive(url: string) {
  return currentPath.value === url || currentPath.value.startsWith(url + '/')
}
</script>

<template>
  <nav
    class="fixed bottom-0 left-0 right-0 z-50 border-t border-border bg-card/95 backdrop-blur-sm md:hidden"
    style="padding-bottom: env(safe-area-inset-bottom, 0px)"
  >
    <div
      class="grid h-14"
      :class="{
        'grid-cols-4': items.length === 4,
        'grid-cols-5': items.length === 5,
        'grid-cols-6': items.length >= 6,
      }"
    >
      <RouterLink
        v-for="item in items"
        :key="item.url"
        :to="item.url"
        class="relative flex flex-col items-center justify-center gap-0.5 transition-colors"
        :class="
          isActive(item.url) ? 'text-prod-yellow' : 'text-muted-foreground active:text-foreground'
        "
      >
        <div class="relative">
          <component :is="item.icon" class="size-5" />
          <span
            v-if="item.url === '/dashboard' && inboxCount > 0"
            class="absolute -right-2 -top-1.5 inline-flex size-4 items-center justify-center rounded-full bg-prod-yellow text-[9px] font-bold leading-none text-prod-yellow-foreground"
          >
            {{ inboxCount > 9 ? '9+' : inboxCount }}
          </span>
        </div>
        <span class="text-[10px] font-medium leading-none">{{ item.title }}</span>
      </RouterLink>
    </div>
  </nav>
</template>
