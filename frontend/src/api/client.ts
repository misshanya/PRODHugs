import axios, { type AxiosError, type InternalAxiosRequestConfig } from 'axios'
import router from '@/router'

const api = axios.create({
  baseURL: '/api/v1',
  headers: {
    'Content-Type': 'application/json',
  },
  withCredentials: true, // send HttpOnly cookies with every request
})

// ── Request interceptor: attach access token ──
api.interceptors.request.use((config) => {
  const token = localStorage.getItem('token')
  if (token) {
    config.headers.Authorization = `Bearer ${token}`
  }
  return config
})

// ── Response interceptor: silent refresh on 401 ──
let isRefreshing = false
let pendingQueue: Array<{
  resolve: (token: string) => void
  reject: (err: unknown) => void
}> = []

function processPendingQueue(token: string | null, error: unknown) {
  for (const p of pendingQueue) {
    if (token) {
      p.resolve(token)
    } else {
      p.reject(error)
    }
  }
  pendingQueue = []
}

const AUTH_PATHS = ['/auth/login', '/auth/register', '/auth/refresh', '/auth/logout']

function isAuthRequest(config: InternalAxiosRequestConfig | undefined): boolean {
  if (!config?.url) return false
  return AUTH_PATHS.some((p) => config.url!.endsWith(p))
}

function forceLogout() {
  localStorage.removeItem('token')
  localStorage.removeItem('user')
  router.push('/login')
}

api.interceptors.response.use(
  (response) => response,
  async (error: AxiosError) => {
    const originalRequest = error.config

    // Only intercept 401s on non-auth endpoints
    if (error.response?.status !== 401 || !originalRequest || isAuthRequest(originalRequest)) {
      return Promise.reject(error)
    }

    // Prevent retrying the same request twice
    if ((originalRequest as InternalAxiosRequestConfig & { _retried?: boolean })._retried) {
      forceLogout()
      return Promise.reject(error)
    }

    // If another refresh is already in-flight, queue this request
    if (isRefreshing) {
      return new Promise<string>((resolve, reject) => {
        pendingQueue.push({ resolve, reject })
      }).then((newToken) => {
        originalRequest.headers.Authorization = `Bearer ${newToken}`
        return api(originalRequest)
      })
    }

    isRefreshing = true
    ;(originalRequest as InternalAxiosRequestConfig & { _retried?: boolean })._retried = true

    try {
      const res = await api.post('/auth/refresh')
      const newToken: string = res.data.token

      localStorage.setItem('token', newToken)
      originalRequest.headers.Authorization = `Bearer ${newToken}`

      processPendingQueue(newToken, null)
      return api(originalRequest)
    } catch (refreshError) {
      processPendingQueue(null, refreshError)
      forceLogout()
      return Promise.reject(refreshError)
    } finally {
      isRefreshing = false
    }
  },
)

export default api

// Auth
export const authApi = {
  register: (username: string, password: string) =>
    api.post('/auth/register', { username, password }),
  login: (username: string, password: string) =>
    api.post('/auth/login', { username, password }),
  logout: () => api.post('/auth/logout'),
  me: () => api.get('/users/me'),
}

// Hugs
export const hugsApi = {
  send: (userId: string) => api.post(`/hugs/${userId}`),
  getCooldown: (userId: string) => api.get(`/hugs/cooldown/${userId}`),
  upgradeCooldown: (userId: string) => api.post(`/hugs/cooldown/${userId}/upgrade`),
  getHistory: () => api.get('/hugs/history'),
  getFeed: (limit = 50) => api.get('/hugs/feed', { params: { limit } }),
  getActivity: () => api.get('/hugs/activity'),
}

// Balance
export const balanceApi = {
  get: () => api.get('/balance'),
  claimDaily: () => api.post('/daily-reward'),
}

// Users
export const usersApi = {
  search: (q = '', limit = 20, offset = 0) =>
    api.get('/users/search', { params: { q, limit, offset } }),
  getProfile: (userId: string) => api.get(`/users/${userId}/profile`),
}

// Leaderboard
export const leaderboardApi = {
  get: (limit = 20, offset = 0) =>
    api.get('/leaderboard', { params: { limit, offset } }),
}
