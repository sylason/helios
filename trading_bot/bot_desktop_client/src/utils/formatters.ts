// trading_bot/bot_desktop_client/src/utils/formatters.ts
import { AssetStatus, PositionSide, StartingPhase } from '../types/bot';

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
 * Lấy nhãn hiển thị tiếng Việt tương ứng với AssetStatus và StartingPhase
 */
export function getStatusLabel(status: AssetStatus, phase?: StartingPhase): string {
  switch (status) {
    case 'INACTIVE':
      return 'TẮT (INACTIVE)';
    case 'STARTING':
      if (phase === 'SYNCING') return 'ĐANG SYNC DATA (33D)';
      if (phase === 'OPTIMIZING') return 'GA OPTIMIZING JIT...';
      return 'KHỞI ĐỘNG (STARTING)...';
    case 'RUNNING_ACTIVE':
      return 'ĐANG GIAO DỊCH (ACTIVE)';
    case 'PAUSED':
      return 'TẠM DỪNG (PAUSED)';
    case 'GRACEFUL_STOPPING':
      return 'CHỜ CHỐT SL (GRACEFUL)';
    case 'STOPPED':
      return 'ĐÃ DỪNG (STOPPED)';
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
    case 'STOPPED':
      return 'secondary';
    case 'STARTING':
    case 'PAUSED':
      return 'warning';
    case 'RUNNING_ACTIVE':
      return 'success';
    case 'GRACEFUL_STOPPING':
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
