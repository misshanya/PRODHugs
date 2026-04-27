<script setup lang="ts">
import { ref, computed } from 'vue'
import { useAuthStore } from '@/stores/auth'
import { validateRegisterForm, parseBackendError, type FieldError } from '@/lib/validation'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group'
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import PasswordRequirements from '@/components/PasswordRequirements.vue'

const auth = useAuthStore()
const username = ref('')
const password = ref('')
const passwordConfirm = ref('')
const gender = ref('')
const serverError = ref('')
const fieldErrors = ref<FieldError[]>([])
const submitted = ref(false)

function errorFor(field: string): string | undefined {
  return fieldErrors.value.find((e) => e.field === field)?.message
}

const hasErrors = computed(() => fieldErrors.value.length > 0)

function validate() {
  fieldErrors.value = validateRegisterForm(username.value, password.value, passwordConfirm.value)
}

async function handleRegister() {
  submitted.value = true
  serverError.value = ''
  validate()
  if (hasErrors.value) return

  try {
    await auth.register(username.value, password.value, gender.value || undefined)
  } catch (e: any) {
    const parsed = parseBackendError(e)
    if (parsed.fieldErrors.length > 0) {
      fieldErrors.value = [...fieldErrors.value, ...parsed.fieldErrors]
    }
    if (parsed.generalError) {
      serverError.value = parsed.generalError
    }
  }
}
</script>

<template>
  <div class="flex min-h-screen items-center justify-center bg-background p-4">
    <Card class="w-full max-w-sm">
      <CardHeader class="text-center">
        <img src="/logo.webp" alt="PROD" class="mx-auto mb-2 size-12 rounded-lg object-contain" />
        <CardTitle class="text-xl">Регистрация</CardTitle>
        <CardDescription>Создай аккаунт в PRODнимашках</CardDescription>
      </CardHeader>
      <CardContent>
        <form @submit.prevent="handleRegister" class="grid gap-4">
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
            <PasswordRequirements :password="password" />
          </div>
          <div class="grid gap-2">
            <Label for="password-confirm">Подтверждение пароля</Label>
            <Input
              id="password-confirm"
              v-model="passwordConfirm"
              type="password"
              placeholder="********"
              :class="{ 'border-destructive': submitted && errorFor('passwordConfirm') }"
              @input="submitted && validate()"
            />
            <p v-if="submitted && errorFor('passwordConfirm')" class="text-xs text-destructive">
              {{ errorFor('passwordConfirm') }}
            </p>
          </div>
          <div class="grid gap-2">
            <Label>Пол <span class="text-muted-foreground text-xs">(необязательно)</span></Label>
            <RadioGroup v-model="gender" class="flex gap-4">
              <div class="flex items-center gap-2">
                <RadioGroupItem id="reg-gender-male" value="male" />
                <Label for="reg-gender-male" class="font-normal cursor-pointer">Мужской</Label>
              </div>
              <div class="flex items-center gap-2">
                <RadioGroupItem id="reg-gender-female" value="female" />
                <Label for="reg-gender-female" class="font-normal cursor-pointer">Женский</Label>
              </div>
            </RadioGroup>
          </div>
          <p v-if="serverError" class="text-sm text-destructive text-center">
            {{ serverError }}
          </p>
          <Button
            type="submit"
            variant="yellow"
            class="w-full rounded-[21px]"
            :disabled="auth.loading"
          >
            {{ auth.loading ? 'Регистрация...' : 'Зарегистрироваться' }}
          </Button>
        </form>
      </CardContent>
      <CardFooter class="justify-center">
        <p class="text-sm text-muted-foreground">
          Уже есть аккаунт?
          <RouterLink
            to="/login"
            class="text-foreground underline underline-offset-4 hover:text-primary"
          >
            Войти
          </RouterLink>
        </p>
      </CardFooter>
    </Card>
  </div>
</template>
