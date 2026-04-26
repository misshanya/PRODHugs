<script setup lang="ts">
import { ref, computed } from 'vue'
import { Heart } from 'lucide-vue-next'
import { useAuthStore } from '@/stores/auth'
import { validateLoginForm, parseBackendError, type FieldError } from '@/lib/validation'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'

const auth = useAuthStore()
const username = ref('')
const password = ref('')
const serverError = ref('')
const fieldErrors = ref<FieldError[]>([])
const submitted = ref(false)

function errorFor(field: string): string | undefined {
  return fieldErrors.value.find((e) => e.field === field)?.message
}

const hasErrors = computed(() => fieldErrors.value.length > 0)

function validate() {
  fieldErrors.value = validateLoginForm(username.value, password.value)
}

async function handleLogin() {
  submitted.value = true
  serverError.value = ''
  validate()
  if (hasErrors.value) return

  try {
    await auth.login(username.value, password.value)
  } catch (e: any) {
    const parsed = parseBackendError(e)
    if (parsed.fieldErrors.length > 0) {
      fieldErrors.value = [...fieldErrors.value, ...parsed.fieldErrors]
    }
    if (parsed.generalError) {
      serverError.value = parsed.generalError
    }
    if (!parsed.generalError && parsed.fieldErrors.length === 0) {
      serverError.value = 'Неверное имя пользователя или пароль'
    }
  }
}
</script>

<template>
  <div class="flex min-h-screen items-center justify-center bg-background p-4">
    <Card class="w-full max-w-sm">
      <CardHeader class="text-center">
        <div class="mx-auto mb-2 flex size-10 items-center justify-center rounded-lg bg-prod-yellow text-prod-yellow-foreground">
          <Heart class="size-5" />
        </div>
        <CardTitle class="text-xl">Вход</CardTitle>
        <CardDescription>Войди в свой аккаунт PRODнимашек</CardDescription>
      </CardHeader>
      <CardContent>
        <form @submit.prevent="handleLogin" class="grid gap-4">
          <div class="grid gap-2">
            <Label for="username">Имя пользователя</Label>
            <Input
              id="username"
              v-model="username"
              type="text"
              placeholder="username"
              :class="{ 'border-destructive': submitted && errorFor('username') }"
              @input="submitted && validate()"
            />
            <p v-if="submitted && errorFor('username')" class="text-xs text-destructive">
              {{ errorFor('username') }}
            </p>
          </div>
          <div class="grid gap-2">
            <Label for="password">Пароль</Label>
            <Input
              id="password"
              v-model="password"
              type="password"
              placeholder="********"
              :class="{ 'border-destructive': submitted && errorFor('password') }"
              @input="submitted && validate()"
            />
            <p v-if="submitted && errorFor('password')" class="text-xs text-destructive">
              {{ errorFor('password') }}
            </p>
          </div>
          <p v-if="serverError" class="text-sm text-destructive text-center">
            {{ serverError }}
          </p>
          <Button type="submit" variant="yellow" class="w-full rounded-[21px]" :disabled="auth.loading">
            {{ auth.loading ? 'Вход...' : 'Войти' }}
          </Button>
        </form>
      </CardContent>
      <CardFooter class="justify-center">
        <p class="text-sm text-muted-foreground">
          Нет аккаунта?
          <RouterLink to="/register" class="text-foreground underline underline-offset-4 hover:text-primary">
            Зарегистрироваться
          </RouterLink>
        </p>
      </CardFooter>
    </Card>
  </div>
</template>
