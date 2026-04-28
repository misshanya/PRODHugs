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

// ── Gender ──
const gender = ref<Gender | ''>((auth.user?.gender as Gender) ?? '')
const savingGender = ref(false)

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
    gender.value = (auth.user?.gender as Gender) ?? ''
    resetPasswordForm()
    fetchBlocked()
  }
})

async function saveGender() {
  savingGender.value = true
  try {
    const payload = gender.value ? { gender: gender.value } : {}
    const res = await usersApi.updateSettings(payload)
    auth.user = res.data
    localStorage.setItem('user', JSON.stringify(res.data))
    toast.success('Пол сохранён')
  } catch (e) {
    const parsed = parseBackendError(e)
    toast.error(parsed.generalError ?? 'Ошибка сохранения')
  } finally {
    savingGender.value = false
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
    <DialogContent class="sm:max-w-md">
      <DialogHeader>
        <DialogTitle>Настройки</DialogTitle>
        <DialogDescription>Управление профилем и безопасностью</DialogDescription>
      </DialogHeader>

      <div class="space-y-6">
        <!-- Gender section -->
        <div class="space-y-3">
          <Label class="text-sm font-medium">Пол</Label>
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
          <Button
            variant="yellow"
            size="sm"
            class="rounded-[21px]"
            :disabled="savingGender"
            @click="saveGender"
          >
            {{ savingGender ? 'Сохранение...' : 'Сохранить пол' }}
          </Button>
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
                  {{ user.username.slice(0, 2).toUpperCase() }}
                </AvatarFallback>
              </Avatar>
              <RouterLink
                :to="`/user/${user.id}`"
                class="min-w-0 flex-1 truncate text-sm font-medium hover:underline"
                @click="open = false"
              >
                {{ user.username }}
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
