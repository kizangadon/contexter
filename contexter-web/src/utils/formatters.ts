/**
 * Shared number formatting utilities.
 *
 * Usage:
 *   formatNumber(1234567)     → "1,234,567"
 *   formatCurrency(1234.5)    → "$1,234.50"
 *   formatPercent(85.3)       → "85.3%"
 */

export function formatNumber(n: number): string {
  return new Intl.NumberFormat().format(n);
}

export function formatCurrency(n: number): string {
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency: 'USD',
    minimumFractionDigits: 2,
  }).format(n);
}

export function formatPercent(n: number): string {
  return `${n.toFixed(1)}%`;
}
