import { clsx, type ClassValue } from 'clsx';
import { twMerge } from 'tailwind-merge';

/**
 * Utility function to merge Tailwind CSS classes with proper precedence
 *
 * This function combines clsx and tailwind-merge to provide intelligent
 * class name merging that respects Tailwind's precedence rules.
 *
 * @param inputs - Class names to merge (strings, objects, arrays)
 * @returns Merged class string
 *
 * @example
 * ```tsx
 * cn('px-4 py-2', 'px-6') // Returns: 'py-2 px-6'
 * cn('text-red-500', someCondition && 'text-blue-500') // Conditionally applies classes
 * cn({ 'bg-blue-500': isActive }, 'px-4') // Object syntax
 * ```
 */
export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}
