<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { useHugsStore } from '@/stores/hugs'
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
  <div class="max-w-3xl mx-auto">
    <h1 class="text-2xl font-bold mb-6">👥 Пользователи</h1>

    <div class="mb-6">
      <input
        v-model="query"
        type="text"
        class="input-field"
        placeholder="🔍 Поиск по имени..."
      />
    </div>

    <div v-if="loading" class="text-center text-indigo-400 py-8">
      Загрузка...
    </div>

    <div v-else-if="users.length === 0" class="text-center text-indigo-400 py-8">
      Пользователи не найдены
    </div>

    <div v-else class="space-y-3">
      <UserCard v-for="user in users" :key="user.id" :user="user" />
    </div>
  </div>
</template>
