import axios from 'axios'
import router from '@/router'

const api = axios.create({
  baseURL: '/api/v1',
  headers: {
    'Content-Type': 'application/json',
  },
})

api.interceptors.request.use((config) => {
  const token = localStorage.getItem('token')
  if (token) {
    config.headers.Authorization = `Bearer ${token}`
  }
  return config
})

api.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response?.status === 401) {
      localStorage.removeItem('token')
      localStorage.removeItem('user')
      router.push('/login')
    }
    return Promise.reject(error)
  },
)

export default api

// Auth
export const authApi = {
  register: (username: string, password: string) =>
    api.post('/auth/register', { username, password }),
  login: (username: string, password: string) =>
    api.post('/auth/login', { username, password }),
  me: () => api.get('/users/me'),
}

// Hugs
export const hugsApi = {
  send: (userId: string) => api.post(`/hugs/${userId}`),
  getCooldown: (userId: string) => api.get(`/hugs/cooldown/${userId}`),
  upgradeCooldown: (userId: string) => api.post(`/hugs/cooldown/${userId}/upgrade`),
  getHistory: () => api.get('/hugs/history'),
  getFeed: (limit = 50) => api.get('/hugs/feed', { params: { limit } }),
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
