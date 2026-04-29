<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick } from 'vue'
import {
  Users,
  ShieldBan,
  ArrowLeft,
  MoreHorizontal,
  Ban,
  ShieldCheck,
  UserPen,
  KeyRound,
  Venus,
  Coins,
} from 'lucide-vue-next'
import { toast } from 'vue-sonner'
import { useAdminStore, type AdminUser } from '@/stores/admin'
import { parseBackendError, type FieldError } from '@/lib/validation'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Avatar, AvatarFallback } from '@/components/ui/avatar'
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'

const admin = useAdminStore()
const loading = ref(true)

// ── Infinite scroll ──
const sentinel = ref<HTMLElement | null>(null)
let observer: IntersectionObserver | null = null

onMounted(async () => {
  await Promise.all([admin.fetchStats(), admin.fetchUsers()])
  loading.value = false

  await nextTick()

  observer = new IntersectionObserver(
    (entries) => {
      if (entries[0]?.isIntersecting && admin.hasMore && !admin.loadingMore) {
        admin.loadMore()
      }
    },
    { rootMargin: '200px' },
  )
  if (sentinel.value) observer.observe(sentinel.value)
})

onUnmounted(() => {
  observer?.disconnect()
})

// ── Dialogs ──
const editingUser = ref<AdminUser | null>(null)

// Username dialog
const usernameDialogOpen = ref(false)
const newUsername = ref('')
const savingUsername = ref(false)
const usernameError = ref('')

function openUsernameDialog(user: AdminUser) {
  editingUser.value = user
  newUsername.value = user.username
  usernameError.value = ''
  usernameDialogOpen.value = true
}

async function saveUsername() {
  if (!editingUser.value || !newUsername.value.trim()) return
  savingUsername.value = true
  usernameError.value = ''
  try {
    await admin.updateUsername(editingUser.value.id, newUsername.value.trim())
    toast.success('Имя пользователя изменено')
    usernameDialogOpen.value = false
  } catch (e) {
    const parsed = parseBackendError(e)
    usernameError.value = parsed.generalError ?? 'Ошибка сохранения'
  } finally {
    savingUsername.value = false
  }
}

// Gender dialog
const genderDialogOpen = ref(false)
const newGender = ref<string>('')
const savingGender = ref(false)

function openGenderDialog(user: AdminUser) {
  editingUser.value = user
  newGender.value = user.gender ?? ''
  genderDialogOpen.value = true
}

async function saveGender() {
  if (!editingUser.value) return
  savingGender.value = true
  try {
    await admin.updateGender(editingUser.value.id, newGender.value || null)
    toast.success('Пол изменён')
    genderDialogOpen.value = false
  } catch {
    toast.error('Ошибка сохранения')
  } finally {
    savingGender.value = false
  }
}

// Password dialog
const passwordDialogOpen = ref(false)
const newPassword = ref('')
const newPasswordConfirm = ref('')
const savingPassword = ref(false)
const passwordErrors = ref<FieldError[]>([])
const passwordServerError = ref('')

function openPasswordDialog(user: AdminUser) {
  editingUser.value = user
  newPassword.value = ''
  newPasswordConfirm.value = ''
  passwordErrors.value = []
  passwordServerError.value = ''
  passwordDialogOpen.value = true
}

function passwordErrorFor(field: string): string | undefined {
  return passwordErrors.value.find((e) => e.field === field)?.message
}

async function savePassword() {
  passwordErrors.value = []
  passwordServerError.value = ''

  if (!newPassword.value) {
    passwordErrors.value.push({ field: 'newPassword', message: 'Введите пароль' })
  } else if (newPassword.value.length < 8) {
    passwordErrors.value.push({ field: 'newPassword', message: 'Минимум 8 символов' })
  }
  if (newPassword.value !== newPasswordConfirm.value) {
    passwordErrors.value.push({ field: 'newPasswordConfirm', message: 'Пароли не совпадают' })
  }
  if (passwordErrors.value.length > 0) return

  if (!editingUser.value) return
  savingPassword.value = true
  try {
    await admin.updatePassword(editingUser.value.id, newPassword.value)
    toast.success('Пароль изменён')
    passwordDialogOpen.value = false
  } catch (e) {
    const parsed = parseBackendError(e)
    passwordServerError.value = parsed.generalError ?? 'Ошибка сохранения'
  } finally {
    savingPassword.value = false
  }
}

// Balance dialog
const balanceDialogOpen = ref(false)
const newBalance = ref(0)
const savingBalance = ref(false)
const balanceError = ref('')

function openBalanceDialog(user: AdminUser) {
  editingUser.value = user
  newBalance.value = user.balance
  balanceError.value = ''
  balanceDialogOpen.value = true
}

async function saveBalance() {
  if (!editingUser.value) return
  if (newBalance.value < 0) {
    balanceError.value = 'Сумма не может быть отрицательной'
    return
  }
  savingBalance.value = true
  balanceError.value = ''
  try {
    await admin.updateBalance(editingUser.value.id, newBalance.value)
    toast.success('Баланс изменён')
    balanceDialogOpen.value = false
  } catch (e) {
    const parsed = parseBackendError(e)
    balanceError.value = parsed.generalError ?? 'Ошибка сохранения'
  } finally {
    savingBalance.value = false
  }
}

// ── Ban / Unban ──
async function toggleBan(user: AdminUser) {
  try {
    if (user.banned_at) {
      await admin.unbanUser(user.id)
      toast.success(`${user.username} разблокирован`)
    } else {
      await admin.banUser(user.id)
      toast.success(`${user.username} заблокирован`)
    }
  } catch (e) {
    const parsed = parseBackendError(e)
    toast.error(parsed.generalError ?? 'Ошибка')
  }
}

function formatDate(dateStr: string): string {
  return new Date(dateStr).toLocaleDateString('ru-RU', {
    day: 'numeric',
    month: 'short',
    year: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  })
}
</script>

<template>
  <div class="mx-auto max-w-3xl space-y-6">
    <!-- Header -->
    <div class="flex items-center gap-3">
      <RouterLink to="/dashboard">
        <Button variant="ghost" size="icon-sm">
          <ArrowLeft class="size-4" />
        </Button>
      </RouterLink>
      <div>
        <h1 class="text-2xl font-semibold tracking-tight">Панель администратора</h1>
        <p class="text-muted-foreground">Управление пользователями</p>
      </div>
    </div>

    <!-- Stats cards -->
    <div v-if="loading" class="grid grid-cols-2 gap-4">
      <Skeleton class="h-24 rounded-lg" />
      <Skeleton class="h-24 rounded-lg" />
    </div>
    <div v-else class="grid grid-cols-2 gap-4">
      <Card>
        <CardHeader class="pb-2">
          <CardTitle class="text-sm font-medium text-muted-foreground"
            >Всего пользователей</CardTitle
          >
        </CardHeader>
        <CardContent>
          <div class="flex items-center gap-2">
            <Users class="size-5 text-prod-yellow" />
            <span class="text-2xl font-bold tabular-nums">{{ admin.stats?.total_users ?? 0 }}</span>
          </div>
        </CardContent>
      </Card>
      <Card>
        <CardHeader class="pb-2">
          <CardTitle class="text-sm font-medium text-muted-foreground">Заблокировано</CardTitle>
        </CardHeader>
        <CardContent>
          <div class="flex items-center gap-2">
            <ShieldBan class="size-5 text-destructive" />
            <span class="text-2xl font-bold tabular-nums">{{
              admin.stats?.banned_users ?? 0
            }}</span>
          </div>
        </CardContent>
      </Card>
    </div>

    <!-- Users list -->
    <div>
      <h2 class="mb-3 text-lg font-medium">Пользователи</h2>

      <div v-if="loading" class="space-y-3">
        <Skeleton v-for="i in 8" :key="i" class="h-16 w-full rounded-lg" />
      </div>

      <div v-else class="space-y-2">
        <div
          v-for="user in admin.users"
          :key="user.id"
          class="flex items-center justify-between rounded-[10px] border p-3 transition-colors hover:bg-accent/50"
        >
          <div class="flex items-center gap-3 min-w-0 flex-1">
            <Avatar class="size-9 shrink-0">
              <AvatarFallback class="text-xs">
                {{ user.username.slice(0, 2).toUpperCase() }}
              </AvatarFallback>
            </Avatar>
            <div class="min-w-0">
              <div class="flex items-center gap-2">
                <p class="text-sm font-medium leading-none truncate">{{ user.username }}</p>
                <Badge
                  v-if="user.role === 'admin'"
                  variant="outline"
                  class="text-[10px] px-1.5 py-0 border-prod-yellow/40 text-prod-yellow"
                >
                  Админ
                </Badge>
                <Badge v-if="user.banned_at" variant="destructive" class="text-[10px] px-1.5 py-0">
                  Бан
                </Badge>
              </div>
              <div class="flex items-center gap-2 mt-1">
                <p class="text-xs text-muted-foreground">
                  {{ user.gender === 'male' ? 'М' : user.gender === 'female' ? 'Ж' : '—' }}
                </p>
                <span class="text-xs text-muted-foreground">·</span>
                <p class="text-xs text-muted-foreground tabular-nums">
                  <Coins class="inline size-3 mr-0.5" />{{ user.balance }}
                </p>
                <p v-if="user.banned_at" class="text-xs text-destructive/70">
                  с {{ formatDate(user.banned_at) }}
                </p>
                <p v-if="user.created_at" class="text-xs text-muted-foreground">
                  рег. {{ formatDate(user.created_at) }}
                </p>
              </div>
            </div>
          </div>

          <DropdownMenu v-if="user.role !== 'admin'">
            <DropdownMenuTrigger as-child>
              <Button variant="ghost" size="icon-sm">
                <MoreHorizontal class="size-4" />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end" class="w-48">
              <DropdownMenuItem @click="toggleBan(user)">
                <template v-if="user.banned_at">
                  <ShieldCheck class="size-4" />
                  Разблокировать
                </template>
                <template v-else>
                  <Ban class="size-4" />
                  Заблокировать
                </template>
              </DropdownMenuItem>
              <DropdownMenuSeparator />
              <DropdownMenuItem @click="openUsernameDialog(user)">
                <UserPen class="size-4" />
                Изменить имя
              </DropdownMenuItem>
              <DropdownMenuItem @click="openGenderDialog(user)">
                <Venus class="size-4" />
                Изменить пол
              </DropdownMenuItem>
              <DropdownMenuItem @click="openPasswordDialog(user)">
                <KeyRound class="size-4" />
                Сменить пароль
              </DropdownMenuItem>
              <DropdownMenuItem @click="openBalanceDialog(user)">
                <Coins class="size-4" />
                Изменить баланс
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>

        <!-- Infinite scroll sentinel -->
        <div ref="sentinel" class="h-1" />

        <div v-if="admin.loadingMore" class="space-y-3 pt-2">
          <Skeleton v-for="i in 3" :key="i" class="h-16 w-full rounded-lg" />
        </div>

        <p
          v-if="!admin.hasMore && admin.users.length > 0"
          class="py-4 text-center text-sm text-muted-foreground"
        >
          Все пользователи загружены
        </p>
      </div>
    </div>

    <!-- Username dialog -->
    <Dialog v-model:open="usernameDialogOpen">
      <DialogContent class="sm:max-w-sm">
        <DialogHeader>
          <DialogTitle>Изменить имя</DialogTitle>
          <DialogDescription> Пользователь: {{ editingUser?.username }} </DialogDescription>
        </DialogHeader>
        <div class="space-y-4">
          <div class="grid gap-1.5">
            <Label for="admin-username">Новое имя</Label>
            <Input
              id="admin-username"
              v-model="newUsername"
              maxlength="32"
              placeholder="username"
              @keydown.enter="saveUsername"
            />
          </div>
          <p v-if="usernameError" class="text-sm text-destructive">{{ usernameError }}</p>
          <Button
            variant="yellow"
            class="w-full rounded-[21px]"
            :disabled="savingUsername"
            @click="saveUsername"
          >
            {{ savingUsername ? 'Сохранение...' : 'Сохранить' }}
          </Button>
        </div>
      </DialogContent>
    </Dialog>

    <!-- Gender dialog -->
    <Dialog v-model:open="genderDialogOpen">
      <DialogContent class="sm:max-w-sm">
        <DialogHeader>
          <DialogTitle>Изменить пол</DialogTitle>
          <DialogDescription> Пользователь: {{ editingUser?.username }} </DialogDescription>
        </DialogHeader>
        <div class="space-y-4">
          <RadioGroup v-model="newGender" class="flex gap-4">
            <div class="flex items-center gap-2">
              <RadioGroupItem id="admin-gender-male" value="male" />
              <Label for="admin-gender-male" class="cursor-pointer font-normal">Мужской</Label>
            </div>
            <div class="flex items-center gap-2">
              <RadioGroupItem id="admin-gender-female" value="female" />
              <Label for="admin-gender-female" class="cursor-pointer font-normal">Женский</Label>
            </div>
          </RadioGroup>
          <Button
            variant="yellow"
            class="w-full rounded-[21px]"
            :disabled="savingGender"
            @click="saveGender"
          >
            {{ savingGender ? 'Сохранение...' : 'Сохранить' }}
          </Button>
        </div>
      </DialogContent>
    </Dialog>

    <!-- Balance dialog -->
    <Dialog v-model:open="balanceDialogOpen">
      <DialogContent class="sm:max-w-sm">
        <DialogHeader>
          <DialogTitle>Изменить баланс</DialogTitle>
          <DialogDescription> Пользователь: {{ editingUser?.username }} </DialogDescription>
        </DialogHeader>
        <div class="space-y-4">
          <div class="grid gap-1.5">
            <Label for="admin-balance">Количество монет</Label>
            <Input
              id="admin-balance"
              v-model.number="newBalance"
              type="number"
              min="0"
              placeholder="0"
              @keydown.enter="saveBalance"
            />
          </div>
          <p v-if="balanceError" class="text-sm text-destructive">{{ balanceError }}</p>
          <Button
            variant="yellow"
            class="w-full rounded-[21px]"
            :disabled="savingBalance"
            @click="saveBalance"
          >
            {{ savingBalance ? 'Сохранение...' : 'Сохранить' }}
          </Button>
        </div>
      </DialogContent>
    </Dialog>

    <!-- Password dialog -->
    <Dialog v-model:open="passwordDialogOpen">
      <DialogContent class="sm:max-w-sm">
        <DialogHeader>
          <DialogTitle>Сменить пароль</DialogTitle>
          <DialogDescription> Пользователь: {{ editingUser?.username }} </DialogDescription>
        </DialogHeader>
        <div class="space-y-4">
          <div class="grid gap-1.5">
            <Label for="admin-new-password">Новый пароль</Label>
            <Input
              id="admin-new-password"
              v-model="newPassword"
              type="password"
              placeholder="********"
              :class="{ 'border-destructive': passwordErrorFor('newPassword') }"
            />
            <p v-if="passwordErrorFor('newPassword')" class="text-xs text-destructive">
              {{ passwordErrorFor('newPassword') }}
            </p>
          </div>
          <div class="grid gap-1.5">
            <Label for="admin-new-password-confirm">Подтвердите пароль</Label>
            <Input
              id="admin-new-password-confirm"
              v-model="newPasswordConfirm"
              type="password"
              placeholder="********"
              :class="{ 'border-destructive': passwordErrorFor('newPasswordConfirm') }"
            />
            <p v-if="passwordErrorFor('newPasswordConfirm')" class="text-xs text-destructive">
              {{ passwordErrorFor('newPasswordConfirm') }}
            </p>
          </div>
          <p v-if="passwordServerError" class="text-sm text-destructive">
            {{ passwordServerError }}
          </p>
          <Button
            variant="yellow"
            class="w-full rounded-[21px]"
            :disabled="savingPassword"
            @click="savePassword"
          >
            {{ savingPassword ? 'Сохранение...' : 'Сменить пароль' }}
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  </div>
</template>
