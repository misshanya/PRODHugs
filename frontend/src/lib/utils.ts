import type { ClassValue } from "clsx"
import { clsx } from "clsx"
import { twMerge } from "tailwind-merge"

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
export function plural(n: number, one: string, few: string, many: string): string {
  const abs = Math.abs(n)
  const mod10 = abs % 10
  const mod100 = abs % 100
  if (mod10 === 1 && mod100 !== 11) return `${n} ${one}`
  if (mod10 >= 2 && mod10 <= 4 && (mod100 < 12 || mod100 > 14)) return `${n} ${few}`
  return `${n} ${many}`
}
