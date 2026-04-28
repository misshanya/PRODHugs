import { ref } from 'vue'

export const accessToken = ref<string | null>(null)

let refreshPromise: Promise<string | null> | null = null

export function getAccessToken(): string | null {
  return accessToken.value
}

export function setAccessToken(token: string | null) {
  accessToken.value = token
}

export function clearAccessToken() {
  accessToken.value = null
}

export async function ensureAccessToken(): Promise<string | null> {
  if (accessToken.value) return accessToken.value
  if (refreshPromise) return refreshPromise

  refreshPromise = fetch('/api/v1/auth/refresh', {
    method: 'POST',
    credentials: 'include',
  })
    .then(async (res) => {
      if (!res.ok) return null
      const data = (await res.json()) as { token?: unknown }
      if (typeof data.token === 'string' && data.token.length > 0) {
        accessToken.value = data.token
        return data.token
      }
      return null
    })
    .catch(() => null)
    .finally(() => {
      refreshPromise = null
    })

  return refreshPromise
}
