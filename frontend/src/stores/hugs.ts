import { defineStore } from 'pinia'
import { ref } from 'vue'
import { hugsApi, balanceApi, leaderboardApi, usersApi } from '@/api/client'

export interface HugFeedItem {
  id: string
  giver_id: string
  receiver_id: string
  giver_username: string
  receiver_username: string
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
  hugs_given: number
  hugs_received: number
  total_hugs: number
  rank: string
  balance?: number
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

  async function sendHug(userId: string) {
    const res = await hugsApi.send(userId)
    await fetchBalance()
    return res.data
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
    loading.value = true
    try {
      const res = await hugsApi.getFeed(limit)
      feed.value = res.data || []
    } finally {
      loading.value = false
    }
  }

  async function fetchLeaderboard(limit = 20, offset = 0) {
    loading.value = true
    try {
      const res = await leaderboardApi.get(limit, offset)
      leaderboard.value = res.data || []
    } finally {
      loading.value = false
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
    fetchBalance,
    claimDailyReward,
    sendHug,
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
