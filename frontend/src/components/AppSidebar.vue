<script setup lang="ts">
import { useRoute } from 'vue-router'
import { computed } from 'vue'
import { LayoutDashboard, Users, Newspaper, Trophy, Shield } from 'lucide-vue-next'
import { useAuthStore } from '@/stores/auth'
import { useHugsStore } from '@/stores/hugs'
import {
  Sidebar,
  SidebarContent,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarRail,
} from '@/components/ui/sidebar'

const route = useRoute()
const auth = useAuthStore()
const hugsStore = useHugsStore()

const items = [
  { title: 'Главная', url: '/dashboard', icon: LayoutDashboard },
  { title: 'Пользователи', url: '/users', icon: Users },
  { title: 'Лента', url: '/feed', icon: Newspaper },
  { title: 'Рейтинг', url: '/leaderboard', icon: Trophy },
]

const isAdmin = computed(() => auth.user?.role === 'admin')
const currentPath = computed(() => route.path)
const inboxCount = computed(() => hugsStore.inboxCount)
</script>

<template>
  <Sidebar collapsible="icon">
    <SidebarHeader class="p-4 group-data-[collapsible=icon]:p-2">
      <div class="flex items-center gap-2 overflow-hidden">
        <img src="/logo.webp" alt="PROD" class="size-10 shrink-0 rounded-lg object-contain" />
        <span class="truncate font-semibold text-foreground"
          ><span class="font-bold">PROD</span>нимашки</span
        >
      </div>
    </SidebarHeader>
    <SidebarContent>
      <SidebarGroup>
        <SidebarGroupLabel class="text-muted-foreground">Навигация</SidebarGroupLabel>
        <SidebarGroupContent>
          <SidebarMenu>
            <SidebarMenuItem v-for="item in items" :key="item.title">
              <SidebarMenuButton
                as-child
                :is-active="currentPath === item.url || currentPath.startsWith(item.url + '/')"
              >
                <RouterLink :to="item.url">
                  <component :is="item.icon" />
                  <span class="flex items-center gap-1.5"
                    >[{{ item.title }}]
                    <span
                      v-if="item.url === '/dashboard' && inboxCount > 0"
                      class="inline-flex size-5 items-center justify-center rounded-full bg-prod-yellow text-[10px] font-bold text-prod-yellow-foreground"
                    >
                      {{ inboxCount > 99 ? '99+' : inboxCount }}
                    </span>
                  </span>
                </RouterLink>
              </SidebarMenuButton>
            </SidebarMenuItem>
          </SidebarMenu>
        </SidebarGroupContent>
      </SidebarGroup>
      <SidebarGroup v-if="isAdmin">
        <SidebarGroupLabel class="text-muted-foreground">Администрирование</SidebarGroupLabel>
        <SidebarGroupContent>
          <SidebarMenu>
            <SidebarMenuItem>
              <SidebarMenuButton
                as-child
                :is-active="currentPath === '/admin' || currentPath.startsWith('/admin/')"
              >
                <RouterLink to="/admin">
                  <Shield />
                  <span>[Админ-панель]</span>
                </RouterLink>
              </SidebarMenuButton>
            </SidebarMenuItem>
          </SidebarMenu>
        </SidebarGroupContent>
      </SidebarGroup>
    </SidebarContent>
    <SidebarRail />
  </Sidebar>
</template>
