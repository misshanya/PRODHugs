import { defineStore } from 'pinia'
import { ref } from 'vue'
import { hugsApi, balanceApi, leaderboardApi, usersApi } from '@/api/client'

export interface HugFeedItem {
  id: string
  giver_id: string
  receiver_id: string
  giver_username: string
  receiver_username: string
  giver_gender?: string | null
  created_at: string
}

export interface PendingHugInboxItem {
  id: string
  giver_id: string
  receiver_id: string
  giver_username: string
  giver_gender?: string | null
  created_at: string
}

export interface OutgoingPendingHug {
  id: string
  giver_id: string
  receiver_id: string
  receiver_username: string
  receiver_gender?: string | null
  created_at: string
}

export interface LeaderboardEntry {
  user_id: string
  username: string
  total_hugs: number
  hugs_given: number
  hugs_received: number
  rank: string
}

export interface UserProfile {
  id: string
  username: string
  role: string
  gender?: string | null
  hugs_given: number
  hugs_received: number
  total_hugs: number
  rank: string
  balance?: number
  mutual_total?: number
  mutual_given?: number
  mutual_received?: number
}

export interface CooldownInfo {
  giver_id: string
  receiver_id: string
  cooldown_seconds: number
  remaining_seconds: number
  can_hug: boolean
}

export interface Balance {
  user_id: string
  amount: number
}

export interface HugActivityItem {
  timestamp: string
  count: number
}

export interface DailyRewardResponse {
  amount: number
  streak_days: number
  new_balance: number
  already_claimed?: boolean
}

export const useHugsStore = defineStore('hugs', () => {
  const balance = ref<Balance | null>(null)
  const feed = ref<HugFeedItem[]>([])
  const leaderboard = ref<LeaderboardEntry[]>([])
  const loading = ref(false)
  const feedLoading = ref(false)
  const leaderboardLoading = ref(false)

  // Inbox / outgoing state
  const inbox = ref<PendingHugInboxItem[]>([])
  const outgoingHug = ref<OutgoingPendingHug | null>(null)
  const inboxCount = ref(0)
  
  // Track timestamps of when a specific user's cooldown needs to be refreshed by HugButton components
  const cooldownRefreshes = ref<Record<string, number>>({})

  function triggerCooldownRefresh(userId: string) {
    cooldownRefreshes.value[userId] = Date.now()
  }

  async function fetchBalance() {
    try {
      const res = await balanceApi.get()
      balance.value = res.data
    } catch {
      // Ignore
    }
  }

  async function claimDailyReward(): Promise<DailyRewardResponse> {
    const res = await balanceApi.claimDaily()
    await fetchBalance()
    return res.data
  }

  async function suggestHug(userId: string) {
    const res = await hugsApi.suggest(userId)
    // The suggest endpoint now returns receiver_username/receiver_gender directly.
    outgoingHug.value = {
      id: res.data.id,
      giver_id: res.data.giver_id,
      receiver_id: res.data.receiver_id,
      receiver_username: res.data.receiver_username,
      receiver_gender: res.data.receiver_gender,
      created_at: res.data.created_at,
    }
    return res.data
  }

  async function acceptHug(hugId: string) {
    const res = await hugsApi.accept(hugId)
    // Remove from inbox
    inbox.value = inbox.value.filter((h) => h.id !== hugId)
    inboxCount.value = Math.max(0, inboxCount.value - 1)
    // Refresh balance (both users get +1 coin)
    await fetchBalance()
    return res.data
  }

  async function declineHug(hugId: string) {
    const res = await hugsApi.decline(hugId)
    // Remove from inbox
    inbox.value = inbox.value.filter((h) => h.id !== hugId)
    inboxCount.value = Math.max(0, inboxCount.value - 1)
    return res.data
  }

  async function cancelOutgoing(hugId: string) {
    const res = await hugsApi.cancel(hugId)
    outgoingHug.value = null
    return res.data
  }

  async function fetchInbox() {
    const res = await hugsApi.getInbox()
    inbox.value = res.data || []
    inboxCount.value = inbox.value.length
    return inbox.value
  }

  async function fetchOutgoing() {
    const res = await hugsApi.getOutgoing()
    outgoingHug.value = res.data || null
    return outgoingHug.value
  }

  async function fetchInboxCount() {
    const res = await hugsApi.getInboxCount()
    inboxCount.value = res.data.count
    return res.data.count
  }

  async function getCooldown(userId: string): Promise<CooldownInfo> {
    const res = await hugsApi.getCooldown(userId)
    return res.data
  }

  async function upgradeCooldown(userId: string): Promise<CooldownInfo> {
    const res = await hugsApi.upgradeCooldown(userId)
    await fetchBalance()
    return res.data
  }

  async function fetchFeed(limit = 50) {
    feedLoading.value = true
    try {
      const res = await hugsApi.getFeed(limit)
      feed.value = res.data || []
    } finally {
      feedLoading.value = false
    }
  }

  async function fetchLeaderboard(limit = 20, offset = 0) {
    leaderboardLoading.value = true
    try {
      const res = await leaderboardApi.get(limit, offset)
      leaderboard.value = res.data || []
    } finally {
      leaderboardLoading.value = false
    }
  }

  async function getHugHistory() {
    const res = await hugsApi.getHistory()
    return res.data || []
  }

  async function getHugActivity(): Promise<HugActivityItem[]> {
    const res = await hugsApi.getActivity()
    return res.data || []
  }

  async function searchUsers(q = '', limit = 20, offset = 0) {
    const res = await usersApi.search(q, limit, offset)
    return res.data || []
  }

  async function getUserProfile(userId: string): Promise<UserProfile> {
    const res = await usersApi.getProfile(userId)
    return res.data
  }

  return {
    balance,
    feed,
    leaderboard,
    loading,
    feedLoading,
    leaderboardLoading,
    inbox,
    outgoingHug,
    inboxCount,
    cooldownRefreshes,
    triggerCooldownRefresh,
    fetchBalance,
    claimDailyReward,
    suggestHug,
    acceptHug,
    declineHug,
    cancelOutgoing,
    fetchInbox,
    fetchOutgoing,
    fetchInboxCount,
    getCooldown,
    upgradeCooldown,
    fetchFeed,
    fetchLeaderboard,
    getHugHistory,
    getHugActivity,
    searchUsers,
    getUserProfile,
  }
})
