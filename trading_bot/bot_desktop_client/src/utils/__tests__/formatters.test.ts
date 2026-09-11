// trading_bot/bot_desktop_client/src/utils/__tests__/formatters.test.ts
import { describe, test, expect } from 'vitest';
import { 
  formatCurrency, 
  formatPercent, 
  getStatusLabel, 
  getStatusBadgeVariant, 
  getPositionBadgeVariant 
} from '../formatters';

describe('Formatters Utils Test Suite', () => {
  test('formatCurrency should format numbers correctly to USD currency', () => {
    expect(formatCurrency(1234.567)).toBe('$1,234.57');
    expect(formatCurrency(0)).toBe('$0.00');
    expect(formatCurrency(-50.5)).toBe('-$50.50');
  });

  test('formatPercent should format positive and negative percentages correctly', () => {
    expect(formatPercent(12.345)).toBe('+12.35%');
    expect(formatPercent(-5.6)).toBe('-5.60%');
    expect(formatPercent(0)).toBe('0.00%');
  });

  test('getStatusLabel should return correct Vietnamese labels for Rust FSM statuses', () => {
    expect(getStatusLabel('INACTIVE')).toBe('TẮT (INACTIVE)');
    expect(getStatusLabel('STARTING', 'SYNCING')).toBe('ĐANG SYNC DATA (33D)');
    expect(getStatusLabel('STARTING', 'OPTIMIZING')).toBe('GA OPTIMIZING JIT...');
    expect(getStatusLabel('STARTING')).toBe('KHỞI ĐỘNG (STARTING)...');
    expect(getStatusLabel('RUNNING_ACTIVE')).toBe('ĐANG GIAO DỊCH (ACTIVE)');
    expect(getStatusLabel('PAUSED')).toBe('TẠM DỪNG (PAUSED)');
    expect(getStatusLabel('GRACEFUL_STOPPING')).toBe('CHỜ CHỐT SL (GRACEFUL)');
    expect(getStatusLabel('STOPPED')).toBe('ĐÃ DỪNG (STOPPED)');
    expect(getStatusLabel('ERROR')).toBe('LỖI HỆ THỐNG (ERROR)');
  });

  test('getStatusBadgeVariant should return correct Bootstrap badge variants for FSM', () => {
    expect(getStatusBadgeVariant('INACTIVE')).toBe('secondary');
    expect(getStatusBadgeVariant('STOPPED')).toBe('secondary');
    expect(getStatusBadgeVariant('STARTING')).toBe('warning');
    expect(getStatusBadgeVariant('PAUSED')).toBe('warning');
    expect(getStatusBadgeVariant('RUNNING_ACTIVE')).toBe('success');
    expect(getStatusBadgeVariant('GRACEFUL_STOPPING')).toBe('dark');
    expect(getStatusBadgeVariant('ERROR')).toBe('danger');
  });

  test('getPositionBadgeVariant should return correct variants for position side', () => {
    expect(getPositionBadgeVariant('LONG')).toBe('success');
    expect(getPositionBadgeVariant('SHORT')).toBe('danger');
    expect(getPositionBadgeVariant('FLAT')).toBe('secondary');
  });
});
