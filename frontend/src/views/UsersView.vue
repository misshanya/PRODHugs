<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { Search } from 'lucide-vue-next'
import { useHugsStore } from '@/stores/hugs'
import { Input } from '@/components/ui/input'
import { Skeleton } from '@/components/ui/skeleton'
import UserCard from '@/components/UserCard.vue'

const hugsStore = useHugsStore()
const query = ref('')
const users = ref<any[]>([])
const loading = ref(false)
let debounceTimer: ReturnType<typeof setTimeout> | null = null

async function search() {
  loading.value = true
  try {
    users.value = await hugsStore.searchUsers(query.value, 30, 0)
  } finally {
    loading.value = false
  }
}

watch(query, () => {
  if (debounceTimer) clearTimeout(debounceTimer)
  debounceTimer = setTimeout(search, 300)
})

onMounted(search)
</script>

<template>
  <div class="mx-auto max-w-2xl space-y-6">
    <div>
      <h1 class="text-2xl font-semibold tracking-tight">Пользователи</h1>
      <p class="text-muted-foreground">Обнимись с кем-нибудь</p>
    </div>

    <div class="relative">
      <Search class="absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
      <Input
        v-model="query"
        type="text"
        class="pl-9"
        placeholder="Поиск по имени..."
        maxlength="64"
      />
    </div>

    <div v-if="loading" class="space-y-3">
      <Skeleton v-for="i in 5" :key="i" class="h-16 w-full rounded-lg" />
    </div>

    <div v-else-if="users.length === 0" class="py-12 text-center text-muted-foreground">
      Такого не нашли(
    </div>

    <div v-else class="space-y-2">
      <UserCard v-for="user in users" :key="user.id" :user="user" />
    </div>
  </div>
</template>
