<script setup lang="ts">
import { RouterView, useRoute } from 'vue-router'
import AppHeader from '@/components/AppHeader.vue'
import AppSidebar from '@/components/AppSidebar.vue'
import { useAuthStore } from '@/stores/auth'
import { computed } from 'vue'

const auth = useAuthStore()
const route = useRoute()

const showLayout = computed(() => {
  return auth.isAuthenticated && !['login', 'register'].includes(route.name as string)
})
</script>

<template>
  <div class="min-h-screen bg-background">
    <template v-if="showLayout">
      <AppHeader />
      <div class="flex">
        <AppSidebar />
        <main class="flex-1 p-6 ml-64 mt-16">
          <RouterView />
        </main>
      </div>
    </template>
    <template v-else>
      <RouterView />
    </template>
  </div>
</template>
