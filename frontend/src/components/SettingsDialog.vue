<script setup lang="ts">
import { ref, watch } from 'vue'
import { toast } from 'vue-sonner'
import { ShieldX } from 'lucide-vue-next'
import { useAuthStore, type Gender } from '@/stores/auth'
import { useHugsStore, type BlockedUser } from '@/stores/hugs'
import { usersApi } from '@/api/client'
import { validateChangePasswordForm, parseBackendError, type FieldError } from '@/lib/validation'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Separator } from '@/components/ui/separator'
import { Avatar, AvatarFallback } from '@/components/ui/avatar'
import PasswordRequirements from '@/components/PasswordRequirements.vue'
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group'

const open = defineModel<boolean>('open', { required: true })

const auth = useAuthStore()
const hugsStore = useHugsStore()

// ── Display name + Gender ──
const displayName = ref(auth.user?.display_name ?? '')
const gender = ref<Gender | ''>((auth.user?.gender as Gender) ?? '')
const savingProfile = ref(false)

// ── Telegram ──
type TelegramStep = 'idle' | 'code_sent' | 'linked'
const telegramStep = ref<TelegramStep>(auth.user?.telegram_id != null ? 'linked' : 'idle')
const telegramId = ref(auth.user?.telegram_id != null ? String(auth.user.telegram_id) : '')
const telegramCode = ref('')
const telegramError = ref('')
const telegramLoading = ref(false)

// ── Blocked users ──
const blockedUsers = ref<BlockedUser[]>([])
const loadingBlocked = ref(false)
const unblockingId = ref<string | null>(null)

async function fetchBlocked() {
  loadingBlocked.value = true
  try {
    blockedUsers.value = await hugsStore.getBlockedUsers()
  } catch {
    // Ignore
  } finally {
    loadingBlocked.value = false
  }
}

async function unblock(userId: string) {
  if (unblockingId.value) return
  unblockingId.value = userId
  try {
    await hugsStore.unblockUser(userId)
    blockedUsers.value = blockedUsers.value.filter((u) => u.id !== userId)
    toast.success('Пользователь разблокирован')
  } catch {
    toast.error('Не удалось разблокировать')
  } finally {
    unblockingId.value = null
  }
}

watch(open, (isOpen) => {
  if (isOpen) {
    displayName.value = auth.user?.display_name ?? ''
    gender.value = (auth.user?.gender as Gender) ?? ''
    telegramStep.value = auth.user?.telegram_id != null ? 'linked' : 'idle'
    telegramId.value = auth.user?.telegram_id != null ? String(auth.user.telegram_id) : ''
    telegramCode.value = ''
    telegramError.value = ''
    resetPasswordForm()
    fetchBlocked()
  }
})

async function saveProfile() {
  savingProfile.value = true
  try {
    const trimmed = displayName.value.trim()
    const payload: { gender?: string; display_name?: string | null } = {}
    if (gender.value) payload.gender = gender.value
    payload.display_name = trimmed || null
    const res = await usersApi.updateSettings(payload)
    auth.user = res.data
    localStorage.setItem('user', JSON.stringify(res.data))
    toast.success('Настройки сохранены')
  } catch (e) {
    const parsed = parseBackendError(e)
    toast.error(parsed.generalError ?? 'Ошибка сохранения')
  } finally {
    savingProfile.value = false
  }
}

function parseTelegramId(): number | null {
  const trimmed = telegramId.value.trim()
  if (trimmed === '') return null
  const n = Number(trimmed)
  if (!Number.isInteger(n) || n <= 0) return null
  return n
}

function telegramErrorFromCode(e: unknown): string {
  const data = (e as { response?: { data?: { code?: string } } })?.response?.data
  const code = data?.code
  if (code === 'INVALID_TELEGRAM_ID')
    return 'Не удалось связаться с этим Telegram ID. Убедитесь, что вы начали диалог с ботом и ID верный.'
  if (code === 'TELEGRAM_ID_TAKEN') return 'Этот Telegram ID уже привязан к другому аккаунту.'
  if (code === 'TELEGRAM_CODE_INVALID') return 'Неверный код подтверждения.'
  if (code === 'TELEGRAM_CODE_EXPIRED') return 'Код подтверждения истёк. Запросите новый.'
  const parsed = parseBackendError(e)
  return parsed.generalError ?? 'Ошибка'
}

async function sendTelegramCode() {
  telegramError.value = ''
  const id = parseTelegramId()
  if (!id) {
    telegramError.value = 'Telegram ID должен быть положительным числом'
    return
  }
  telegramLoading.value = true
  try {
    await usersApi.sendTelegramCode(id)
    telegramStep.value = 'code_sent'
    telegramCode.value = ''
  } catch (e) {
    telegramError.value = telegramErrorFromCode(e)
  } finally {
    telegramLoading.value = false
  }
}

async function verifyTelegramCode() {
  telegramError.value = ''
  const id = parseTelegramId()
  if (!id) return
  const code = telegramCode.value.trim()
  if (code.length !== 6) {
    telegramError.value = 'Введите 6-значный код'
    return
  }
  telegramLoading.value = true
  try {
    const res = await usersApi.verifyTelegramCode(id, code)
    auth.user = res.data
    localStorage.setItem('user', JSON.stringify(res.data))
    telegramStep.value = 'linked'
    toast.success('Telegram привязан')
  } catch (e) {
    telegramError.value = telegramErrorFromCode(e)
  } finally {
    telegramLoading.value = false
  }
}

async function unlinkTelegram() {
  telegramError.value = ''
  telegramLoading.value = true
  try {
    const res = await usersApi.unlinkTelegram()
    auth.user = res.data
    localStorage.setItem('user', JSON.stringify(res.data))
    telegramStep.value = 'idle'
    telegramId.value = ''
    telegramCode.value = ''
    toast.success('Telegram отвязан')
  } catch (e) {
    telegramError.value = telegramErrorFromCode(e)
  } finally {
    telegramLoading.value = false
  }
}

// ── Password ──
const oldPassword = ref('')
const newPassword = ref('')
const newPasswordConfirm = ref('')
const passwordErrors = ref<FieldError[]>([])
const passwordServerError = ref('')
const savingPassword = ref(false)
const passwordSubmitted = ref(false)

function resetPasswordForm() {
  oldPassword.value = ''
  newPassword.value = ''
  newPasswordConfirm.value = ''
  passwordErrors.value = []
  passwordServerError.value = ''
  passwordSubmitted.value = false
}

function passwordErrorFor(field: string): string | undefined {
  return passwordErrors.value.find((e) => e.field === field)?.message
}

function validatePasswordForm() {
  passwordErrors.value = validateChangePasswordForm(
    oldPassword.value,
    newPassword.value,
    newPasswordConfirm.value,
  )
}

async function savePassword() {
  passwordSubmitted.value = true
  passwordServerError.value = ''
  validatePasswordForm()
  if (passwordErrors.value.length > 0) return

  savingPassword.value = true
  try {
    await usersApi.changePassword(oldPassword.value, newPassword.value)
    toast.success('Пароль изменён')
    resetPasswordForm()
  } catch (e) {
    const parsed = parseBackendError(e)
    if (parsed.fieldErrors.length > 0) {
      passwordErrors.value = [...passwordErrors.value, ...parsed.fieldErrors]
    }
    if (parsed.generalError) {
      passwordServerError.value = parsed.generalError
    }
  } finally {
    savingPassword.value = false
  }
}
</script>

<template>
  <Dialog v-model:open="open">
    <DialogContent class="sm:max-w-md max-h-[calc(100dvh-2rem)] flex flex-col overflow-hidden">
      <DialogHeader>
        <DialogTitle>Настройки</DialogTitle>
        <DialogDescription>Управление профилем и безопасностью</DialogDescription>
      </DialogHeader>

      <div class="-mx-4 flex-1 space-y-6 overflow-y-auto overscroll-contain px-4 pb-1">
        <!-- Profile section -->
        <div class="space-y-3">
          <Label class="text-sm font-medium">Профиль</Label>
          <div class="grid gap-1.5">
            <Label for="settings-display-name" class="text-xs text-muted-foreground"
              >Отображаемое имя</Label
            >
            <Input
              id="settings-display-name"
              v-model="displayName"
              maxlength="32"
              placeholder="Как тебя называть"
            />
            <p class="text-[11px] text-muted-foreground">
              Оставь пустым, чтобы использовать имя пользователя.
            </p>
          </div>
          <div class="grid gap-1.5">
            <Label class="text-xs text-muted-foreground">Пол</Label>
            <RadioGroup v-model="gender" class="flex gap-4">
              <div class="flex items-center gap-2">
                <RadioGroupItem id="gender-male" value="male" />
                <Label for="gender-male" class="font-normal cursor-pointer">Мужской</Label>
              </div>
              <div class="flex items-center gap-2">
                <RadioGroupItem id="gender-female" value="female" />
                <Label for="gender-female" class="font-normal cursor-pointer">Женский</Label>
              </div>
            </RadioGroup>
          </div>
          <Button
            variant="yellow"
            size="sm"
            class="rounded-[21px]"
            :disabled="savingProfile"
            @click="saveProfile"
          >
            {{ savingProfile ? 'Сохранение...' : 'Сохранить' }}
          </Button>
        </div>

        <Separator />

        <!-- Telegram section -->
        <div class="space-y-3">
          <Label class="text-sm font-medium">Telegram уведомления</Label>

          <!-- State C: Already linked -->
          <div v-if="telegramStep === 'linked'" class="space-y-2">
            <div
              class="flex items-center gap-2 rounded-md border border-green-800/40 bg-green-950/30 px-3 py-2 text-sm"
            >
              <span class="text-green-400">✓</span>
              <span>Telegram привязан (ID: {{ auth.user?.telegram_id }})</span>
            </div>
            <Button
              variant="outline"
              size="sm"
              class="rounded-[21px]"
              :disabled="telegramLoading"
              @click="unlinkTelegram"
            >
              {{ telegramLoading ? 'Отвязка...' : 'Отвязать Telegram' }}
            </Button>
          </div>

          <!-- State A: Not linked — enter ID -->
          <div v-else-if="telegramStep === 'idle'" class="grid gap-1.5">
            <Label for="settings-telegram-id" class="text-xs text-muted-foreground"
              >Telegram ID</Label
            >
            <Input
              id="settings-telegram-id"
              v-model="telegramId"
              inputmode="numeric"
              placeholder="Например: 123456789"
              :class="{ 'border-destructive': telegramError }"
            />
            <div class="space-y-1 text-[11px] text-muted-foreground">
              <p>
                1. Начните диалог с ботом
                <a
                  href="https://t.me/prodhugsbot"
                  target="_blank"
                  class="text-primary underline underline-offset-2"
                  >@prodhugsbot</a
                >
                (нажмите Start).
              </p>
              <p>
                2. Узнайте свой ID у
                <a
                  href="https://t.me/userinfobot"
                  target="_blank"
                  class="text-primary underline underline-offset-2"
                  >@userinfobot</a
                >
                и введите его выше.
              </p>
            </div>
            <Button
              variant="yellow"
              size="sm"
              class="mt-1 rounded-[21px]"
              :disabled="telegramLoading || !telegramId.trim()"
              @click="sendTelegramCode"
            >
              {{ telegramLoading ? 'Отправка...' : 'Отправить код' }}
            </Button>
          </div>

          <!-- State B: Code sent — enter verification code -->
          <div v-else-if="telegramStep === 'code_sent'" class="grid gap-1.5">
            <p class="text-xs text-muted-foreground">
              Код отправлен на Telegram ID <strong>{{ telegramId }}</strong>
            </p>
            <Label for="settings-telegram-code" class="text-xs text-muted-foreground"
              >Код подтверждения</Label
            >
            <Input
              id="settings-telegram-code"
              v-model="telegramCode"
              inputmode="numeric"
              maxlength="6"
              placeholder="6-значный код"
              :class="{ 'border-destructive': telegramError }"
            />
            <div class="flex items-center gap-3">
              <Button
                variant="yellow"
                size="sm"
                class="rounded-[21px]"
                :disabled="telegramLoading || telegramCode.trim().length !== 6"
                @click="verifyTelegramCode"
              >
                {{ telegramLoading ? 'Проверка...' : 'Подтвердить' }}
              </Button>
              <button
                type="button"
                class="text-xs text-muted-foreground underline underline-offset-2 hover:text-foreground"
                :disabled="telegramLoading"
                @click="sendTelegramCode"
              >
                Отправить заново
              </button>
              <button
                type="button"
                class="text-xs text-muted-foreground underline underline-offset-2 hover:text-foreground"
                @click="telegramStep = 'idle'; telegramError = ''; telegramCode = ''"
              >
                Назад
              </button>
            </div>
          </div>

          <p v-if="telegramError" class="text-xs text-destructive">
            {{ telegramError }}
          </p>
        </div>

        <Separator />

        <!-- Password section -->
        <div class="space-y-3">
          <Label class="text-sm font-medium">Смена пароля</Label>
          <div class="grid gap-3">
            <div class="grid gap-1.5">
              <Label for="old-password" class="text-xs text-muted-foreground">Текущий пароль</Label>
              <Input
                id="old-password"
                v-model="oldPassword"
                type="password"
                placeholder="********"
                :class="{
                  'border-destructive': passwordSubmitted && passwordErrorFor('oldPassword'),
                }"
                @input="passwordSubmitted && validatePasswordForm()"
              />
              <p
                v-if="passwordSubmitted && passwordErrorFor('oldPassword')"
                class="text-xs text-destructive"
              >
                {{ passwordErrorFor('oldPassword') }}
              </p>
            </div>
            <div class="grid gap-1.5">
              <Label for="new-password" class="text-xs text-muted-foreground">Новый пароль</Label>
              <Input
                id="new-password"
                v-model="newPassword"
                type="password"
                placeholder="********"
                :class="{
                  'border-destructive': passwordSubmitted && passwordErrorFor('newPassword'),
                }"
                @input="passwordSubmitted && validatePasswordForm()"
              />
              <p
                v-if="passwordSubmitted && passwordErrorFor('newPassword')"
                class="text-xs text-destructive"
              >
                {{ passwordErrorFor('newPassword') }}
              </p>
              <PasswordRequirements :password="newPassword" />
            </div>
            <div class="grid gap-1.5">
              <Label for="new-password-confirm" class="text-xs text-muted-foreground"
                >Подтвердите новый пароль</Label
              >
              <Input
                id="new-password-confirm"
                v-model="newPasswordConfirm"
                type="password"
                placeholder="********"
                :class="{
                  'border-destructive': passwordSubmitted && passwordErrorFor('newPasswordConfirm'),
                }"
                @input="passwordSubmitted && validatePasswordForm()"
              />
              <p
                v-if="passwordSubmitted && passwordErrorFor('newPasswordConfirm')"
                class="text-xs text-destructive"
              >
                {{ passwordErrorFor('newPasswordConfirm') }}
              </p>
            </div>
          </div>
          <p v-if="passwordServerError" class="text-sm text-destructive">
            {{ passwordServerError }}
          </p>
          <Button
            variant="yellow"
            size="sm"
            class="rounded-[21px]"
            :disabled="savingPassword"
            @click="savePassword"
          >
            {{ savingPassword ? 'Сохранение...' : 'Сменить пароль' }}
          </Button>
        </div>

        <Separator />

        <!-- Blocked users section -->
        <div class="space-y-3">
          <Label class="text-sm font-medium">Заблокированные пользователи</Label>
          <div v-if="loadingBlocked" class="py-4 text-center text-sm text-muted-foreground">
            Загрузка...
          </div>
          <div
            v-else-if="blockedUsers.length === 0"
            class="py-4 text-center text-sm text-muted-foreground"
          >
            Нет заблокированных пользователей
          </div>
          <div v-else class="space-y-2">
            <div
              v-for="user in blockedUsers"
              :key="user.id"
              class="flex items-center gap-3 rounded-md border px-3 py-2"
            >
              <Avatar class="size-7 shrink-0">
                <AvatarFallback class="text-[10px]">
                  {{ (user.display_name || user.username).slice(0, 2).toUpperCase() }}
                </AvatarFallback>
              </Avatar>
              <RouterLink
                :to="`/user/${user.id}`"
                class="min-w-0 flex-1 truncate text-sm font-medium hover:underline"
                @click="open = false"
              >
                {{ user.display_name || user.username }}
              </RouterLink>
              <Button
                variant="ghost"
                size="sm"
                class="shrink-0 gap-1 text-xs"
                :disabled="unblockingId === user.id"
                @click="unblock(user.id)"
              >
                <ShieldX class="size-3.5" />
                Разблокировать
              </Button>
            </div>
          </div>
        </div>
      </div>
    </DialogContent>
  </Dialog>
</template>
