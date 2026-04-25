<script setup lang="ts">
import { ref } from 'vue'
import { useAuthStore } from '@/stores/auth'

const auth = useAuthStore()
const username = ref('')
const password = ref('')
const errorMsg = ref('')

async function handleLogin() {
  errorMsg.value = ''
  try {
    await auth.login(username.value, password.value)
  } catch (e: any) {
    errorMsg.value = e.response?.data?.message || 'Неверное имя пользователя или пароль'
  }
}
</script>

<template>
  <div class="min-h-screen flex items-center justify-center bg-background">
    <div class="w-full max-w-md">
      <div class="text-center mb-8">
        <span class="text-6xl">🤗</span>
        <h1 class="text-3xl font-bold mt-4 bg-gradient-to-r from-primary-light to-accent bg-clip-text text-transparent">
          Hugs as a Service
        </h1>
        <p class="text-indigo-400 mt-2">Платформа для объятий</p>
      </div>

      <div class="card">
        <h2 class="text-xl font-bold mb-6 text-center">Вход</h2>

        <form @submit.prevent="handleLogin" class="space-y-4">
          <div>
            <label class="block text-sm text-indigo-300 mb-1">Имя пользователя</label>
            <input
              v-model="username"
              type="text"
              class="input-field"
              placeholder="Введите имя пользователя"
              required
            />
          </div>

          <div>
            <label class="block text-sm text-indigo-300 mb-1">Пароль</label>
            <input
              v-model="password"
              type="password"
              class="input-field"
              placeholder="Введите пароль"
              required
            />
          </div>

          <div v-if="errorMsg" class="text-red-400 text-sm text-center bg-red-900/20 rounded-lg p-2">
            {{ errorMsg }}
          </div>

          <button
            type="submit"
            :disabled="auth.loading"
            class="btn-primary w-full"
          >
            {{ auth.loading ? 'Загрузка...' : 'Войти' }}
          </button>
        </form>

        <p class="text-center text-sm text-indigo-400 mt-4">
          Нет аккаунта?
          <RouterLink to="/register" class="text-primary-light hover:underline">Зарегистрироваться</RouterLink>
        </p>
      </div>
    </div>
  </div>
</template>
