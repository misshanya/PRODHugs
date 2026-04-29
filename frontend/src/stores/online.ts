import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useOnlineStore = defineStore('online', () => {
  const userIds = ref<Set<string>>(new Set())

  function setUsers(ids: string[]) {
    userIds.value = new Set(ids)
  }

  function isOnline(userId: string): boolean {
    return userIds.value.has(userId)
  }

  function clear() {
    userIds.value = new Set()
  }

  return { userIds, setUsers, isOnline, clear }
})
