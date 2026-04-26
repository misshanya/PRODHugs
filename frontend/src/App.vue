<script setup lang="ts">
import { RouterView, useRoute } from 'vue-router'
import { computed } from 'vue'
import { useAuthStore } from '@/stores/auth'
import { SidebarProvider, SidebarInset, SidebarTrigger } from '@/components/ui/sidebar'
import { Separator } from '@/components/ui/separator'
import { Toaster } from '@/components/ui/sonner'
import AppSidebar from '@/components/AppSidebar.vue'
import AppHeader from '@/components/AppHeader.vue'

const auth = useAuthStore()
const route = useRoute()

const showLayout = computed(() => {
  return auth.isAuthenticated && !['login', 'register'].includes(route.name as string)
})
</script>

<template>
  <template v-if="showLayout">
    <SidebarProvider>
      <AppSidebar />
      <SidebarInset>
        <header class="flex h-14 shrink-0 items-center gap-2 border-b px-4">
          <SidebarTrigger class="-ml-1" />
          <Separator orientation="vertical" class="mr-2 !h-4" />
          <AppHeader />
        </header>
        <main class="flex-1 p-3 sm:p-6">
          <RouterView />
        </main>
      </SidebarInset>
    </SidebarProvider>
  </template>
  <template v-else>
    <RouterView />
  </template>
  <Toaster position="top-right" />
</template>
