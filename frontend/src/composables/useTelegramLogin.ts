import { ref, onUnmounted } from 'vue'
import { type AxiosError } from 'axios'
import { authApi } from '@/api/client'
import { setAccessToken } from '@/lib/token'
import { useAuthStore } from '@/stores/auth'
import router from '@/router'

export function useTelegramLogin() {
  const auth = useAuthStore()

  const telegramPolling = ref(false)
  const telegramError = ref<string | null>(null)
  const telegramLoading = ref(false)

  let pollInterval: ReturnType<typeof setInterval> | null = null
  let pollToken: string | null = null
  let pollAttempts = 0
  const MAX_POLL_ATTEMPTS = 150 // 5 minutes at 2-second intervals

  function stopPolling() {
    if (pollInterval) {
      clearInterval(pollInterval)
      pollInterval = null
    }
    telegramPolling.value = false
    pollToken = null
    pollAttempts = 0
  }

  async function startTelegramLogin() {
    telegramError.value = null
    telegramLoading.value = true

    try {
      const res = await authApi.initTelegramLogin()
      const { bot_url, poll_token } = res.data

      pollToken = poll_token
      pollAttempts = 0
      telegramPolling.value = true

      // Open bot in a new tab
      window.open(bot_url, '_blank')

      // Start polling
      pollInterval = setInterval(async () => {
        if (!pollToken) {
          stopPolling()
          return
        }

        pollAttempts++
        if (pollAttempts >= MAX_POLL_ATTEMPTS) {
          stopPolling()
          telegramError.value = 'Время ожидания истекло. Попробуйте снова'
          return
        }

        try {
          const pollRes = await authApi.pollTelegramLogin(pollToken)

          if (pollRes.status === 200) {
            // Login successful
            stopPolling()
            const data = pollRes.data
            auth.token = data.token
            auth.user = data.user
            setAccessToken(data.token)
            localStorage.setItem('user', JSON.stringify(data.user))
            await router.push('/dashboard')
          }
          // 202 = still pending, keep polling
        } catch (err: unknown) {
          const axiosErr = err as AxiosError<{ message?: string }>
          if (axiosErr.response?.status === 403) {
            stopPolling()
            telegramError.value = axiosErr.response?.data?.message || 'Аккаунт заблокирован'
          } else if (axiosErr.response?.status === 404) {
            stopPolling()
            telegramError.value = 'Сессия истекла. Попробуйте снова'
          }
          // Other errors: keep polling (transient network issues)
        }
      }, 2000)
    } catch (err: unknown) {
      const axiosErr = err as AxiosError
      if (axiosErr.response?.status === 503) {
        telegramError.value = 'Вход через Telegram недоступен'
      } else {
        telegramError.value = 'Не удалось начать вход через Telegram'
      }
    } finally {
      telegramLoading.value = false
    }
  }

  function cancelTelegramLogin() {
    stopPolling()
    telegramError.value = null
  }

  onUnmounted(() => {
    stopPolling()
  })

  return {
    telegramPolling,
    telegramError,
    telegramLoading,
    startTelegramLogin,
    cancelTelegramLogin,
  }
}
