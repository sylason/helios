// trading_bot/bot_desktop_client/src/utils/formatters.ts
import { AssetStatus, PositionSide } from '../types/bot';

/**
 * Định dạng số tiền tiền tệ USD (ví dụ: $1,234.56 hoặc -$50.50)
 */
export function formatCurrency(amount: number): string {
  const isNegative = amount < 0;
  const absAmount = Math.abs(amount);
  const formatted = new Intl.NumberFormat('en-US', {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  }).format(absAmount);
  return `${isNegative ? '-' : ''}$${formatted}`;
}

/**
 * Định dạng phần trăm với dấu + hoặc - (ví dụ: +12.34% hoặc -5.67%)
 */
export function formatPercent(value: number): string {
  const prefix = value > 0 ? '+' : '';
  return `${prefix}${value.toFixed(2)}%`;
}

/**
 * Lấy nhãn hiển thị tiếng Việt tương ứng với AssetStatus
 */
export function getStatusLabel(status: AssetStatus): string {
  switch (status) {
    case 'INACTIVE':
      return 'TẮT (INACTIVE)';
    case 'SYNCING_DATA':
      return 'ĐANG SYNC DATA (60D)';
    case 'GA_OPTIMIZING':
      return 'GA OPTIMIZING...';
    case 'READY_TRADING':
      return 'ĐANG TRADING (LIVE)';
    case 'GRACEFUL_EXIT':
      return 'GRACEFUL EXITING...';
    case 'ERROR':
      return 'LỖI HỆ THỐNG (ERROR)';
    default:
      return 'KHÔNG XÁC ĐỊNH';
  }
}

/**
 * Lấy CSS Bootstrap variant tương ứng với AssetStatus
 */
export function getStatusBadgeVariant(status: AssetStatus): string {
  switch (status) {
    case 'INACTIVE':
      return 'secondary';
    case 'SYNCING_DATA':
      return 'warning';
    case 'GA_OPTIMIZING':
      return 'info';
    case 'READY_TRADING':
      return 'success';
    case 'GRACEFUL_EXIT':
      return 'dark';
    case 'ERROR':
      return 'danger';
    default:
      return 'secondary';
  }
}

/**
 * Lấy Bootstrap Badge Variant cho PositionSide
 */
export function getPositionBadgeVariant(side: PositionSide): string {
  switch (side) {
    case 'LONG':
      return 'success';
    case 'SHORT':
      return 'danger';
    case 'FLAT':
    default:
      return 'secondary';
  }
}
