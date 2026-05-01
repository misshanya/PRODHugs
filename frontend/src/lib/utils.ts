import type { ClassValue } from 'clsx'
import { clsx } from 'clsx'
import { twMerge } from 'tailwind-merge'

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

/**
 * Russian pluralization.
 * Returns the correct form for a number:
 *   plural(1, 'монета', 'монеты', 'монет')  → '1 монета'
 *   plural(3, 'монета', 'монеты', 'монет')  → '3 монеты'
 *   plural(5, 'монета', 'монеты', 'монет')  → '5 монет'
 *   plural(21, 'монета', 'монеты', 'монет') → '21 монета'
 */
/**
 * Gender-aware verb form for Russian.
 * Returns 'обнял' for male, 'обняла' for female, 'обнял(а)' when unknown.
 */
export function hugVerb(gender?: string | null): string {
  if (gender === 'male') return 'обнял'
  if (gender === 'female') return 'обняла'
  return 'обнял(а)'
}

/**
 * Returns the full feed phrase with hug type integrated naturally.
 * e.g. "тепло обнял(а)", "обнял(а) по-медвежьи"
 */
export function hugFeedPhrase(gender?: string | null, hugType?: string): string {
  const verb = hugVerb(gender)
  switch (hugType) {
    case 'bear':
      return `${verb} по-медвежьи`
    case 'warm':
      return `тепло ${verb}`
    case 'group':
      return `${verb} вместе со всеми`
    case 'soul':
      return `по-душевному ${verb}`
    default:
      return verb
  }
}

export function suggestVerb(gender?: string | null): string {
  if (gender === 'male') return 'предложил'
  if (gender === 'female') return 'предложила'
  return 'предложил(а)'
}

/**
 * Returns a natural suggestion phrase for the inbox.
 * e.g. "хочет обнять тебя по-медвежьи", "хочет тепло тебя обнять"
 */
export function hugSuggestionPhrase(hugType?: string): string {
  switch (hugType) {
    case 'bear':
      return 'хочет обнять тебя по-медвежьи'
    case 'warm':
      return 'хочет тепло тебя обнять'
    case 'group':
      return 'хочет обнять тебя вместе со всеми'
    case 'soul':
      return 'хочет обнять тебя по-душевному'
    default:
      return 'предлагает обняться'
  }
}

/**
 * Returns a toast message for a completed hug.
 * e.g. "Медвежьи обнимашки с X приняты!", "Обнимашки с X приняты!"
 */
export function hugCompletedToast(username: string, hugType?: string): string {
  switch (hugType) {
    case 'bear':
      return `Медвежьи обнимашки с ${username} приняты!`
    case 'warm':
      return `Тёплые обнимашки с ${username} приняты!`
    case 'group':
      return `Групповые обнимашки с ${username} приняты!`
    case 'soul':
      return `Душевные обнимашки с ${username} приняты!`
    default:
      return `Обнимашки с ${username} приняты!`
  }
}

/** Map hug type to a Russian label. */
export function hugTypeLabel(hugType: string): string {
  switch (hugType) {
    case 'bear':
      return 'Медвежьи'
    case 'group':
      return 'Групповые'
    case 'warm':
      return 'Тёплые'
    case 'soul':
      return 'Душевные'
    default:
      return 'Обычные'
  }
}

export function plural(n: number, one: string, few: string, many: string): string {
  const abs = Math.abs(n)
  const mod10 = abs % 10
  const mod100 = abs % 100
  if (mod10 === 1 && mod100 !== 11) return `${n} ${one}`
  if (mod10 >= 2 && mod10 <= 4 && (mod100 < 12 || mod100 > 14)) return `${n} ${few}`
  return `${n} ${many}`
}
