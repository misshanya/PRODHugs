<script setup lang="ts">
import { ref } from 'vue'
import { useAuthStore } from '@/stores/auth'

const auth = useAuthStore()
const username = ref('')
const password = ref('')
const passwordConfirm = ref('')
const errorMsg = ref('')

async function handleRegister() {
  errorMsg.value = ''
  if (password.value !== passwordConfirm.value) {
    errorMsg.value = 'Пароли не совпадают'
    return
  }
  if (password.value.length < 4) {
    errorMsg.value = 'Пароль должен быть не менее 4 символов'
    return
  }
  try {
    await auth.register(username.value, password.value)
  } catch (e: any) {
    errorMsg.value = e.response?.data?.message || 'Ошибка регистрации'
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
        <p class="text-indigo-400 mt-2">Присоединяйтесь к сообществу обнимашек!</p>
      </div>

      <div class="card">
        <h2 class="text-xl font-bold mb-6 text-center">Регистрация</h2>

        <form @submit.prevent="handleRegister" class="space-y-4">
          <div>
            <label class="block text-sm text-indigo-300 mb-1">Имя пользователя</label>
            <input
              v-model="username"
              type="text"
              class="input-field"
              placeholder="Выберите имя пользователя"
              required
            />
          </div>

          <div>
            <label class="block text-sm text-indigo-300 mb-1">Пароль</label>
            <input
              v-model="password"
              type="password"
              class="input-field"
              placeholder="Придумайте пароль"
              required
            />
          </div>

          <div>
            <label class="block text-sm text-indigo-300 mb-1">Подтверждение пароля</label>
            <input
              v-model="passwordConfirm"
              type="password"
              class="input-field"
              placeholder="Повторите пароль"
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
            {{ auth.loading ? 'Загрузка...' : 'Зарегистрироваться' }}
          </button>
        </form>

        <p class="text-center text-sm text-indigo-400 mt-4">
          Уже есть аккаунт?
          <RouterLink to="/login" class="text-primary-light hover:underline">Войти</RouterLink>
        </p>
      </div>
    </div>
  </div>
</template>
