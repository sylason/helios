// trading_bot/bot_desktop_client/src/services/botService.ts
import { invoke } from '@tauri-apps/api/core';
import { AccountState, AssetState, SystemHealth } from '../types/bot';

/**
 * Kiểm tra xem app có đang chạy trong môi trường Desktop Tauri hay không
 */
export function isTauriEnvironment(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

/**
 * Danh sách Cặp tiền Mặc định cho bản Demo
 */
export const INITIAL_MOCK_ASSETS: AssetState[] = [
  {
    symbol: 'ETHUSDT',
    marketType: 'FUTURES 20x',
    status: 'INACTIVE',
    strategyName: 'Helios Strategy V3',
    fitnessReturnPct: 125.16,
    fitnessMaxDdPct: -12.34,
    position: {
      side: 'FLAT',
      qty: 0,
      entryPrice: 0,
      unrealizedPnl: 0,
      unrealizedPnlPct: 0,
    },
    recentLogs: [
      'System initialized. Waiting for user activation.',
    ],
  },
  {
    symbol: 'BTCUSDT',
    marketType: 'FUTURES 20x',
    status: 'INACTIVE',
    strategyName: 'Helios Strategy V3',
    fitnessReturnPct: 98.45,
    fitnessMaxDdPct: -10.15,
    position: {
      side: 'FLAT',
      qty: 0,
      entryPrice: 0,
      unrealizedPnl: 0,
      unrealizedPnlPct: 0,
    },
    recentLogs: [
      'System initialized. Standby mode.',
    ],
  },
  {
    symbol: 'SOLUSDT',
    marketType: 'FUTURES 20x',
    status: 'INACTIVE',
    strategyName: 'Helios Strategy V3',
    fitnessReturnPct: 142.80,
    fitnessMaxDdPct: -15.20,
    position: {
      side: 'FLAT',
      qty: 0,
      entryPrice: 0,
      unrealizedPnl: 0,
      unrealizedPnlPct: 0,
    },
    recentLogs: [
      'System initialized. Standby mode.',
    ],
  },
];

export const INITIAL_MOCK_ACCOUNT: AccountState = {
  walletBalance: 10000.00,
  availableBalance: 10000.00,
  totalUnrealizedPnl: 0.00,
  activeAssetsCount: 0,
};

export const INITIAL_MOCK_HEALTH: SystemHealth = {
  apiWeightUsed: 45,
  apiWeightMax: 2400,
  wsConnected: true,
  latencyMs: 38,
};

/**
 * Service giao tiếp IPC giữa React Client & Rust BotEngine
 */
export class BotService {
  /**
   * Kích hoạt Bot cho một Cặp giao dịch (ACTIVATE)
   */
  static async activateAsset(symbol: string): Promise<void> {
    if (isTauriEnvironment()) {
      try {
        await invoke('cmd_activate_asset', { asset: symbol });
      } catch (err) {
        console.error(`[Tauri IPC Error] Activate ${symbol}:`, err);
        throw err;
      }
    } else {
      console.log(`[Browser Mock IPC] Activate asset: ${symbol}`);
    }
  }

  /**
   * Dừng an toàn (GRACEFUL TERMINATE)
   */
  static async gracefulTerminateAsset(symbol: string): Promise<void> {
    if (isTauriEnvironment()) {
      try {
        await invoke('cmd_terminate_graceful', { asset: symbol });
      } catch (err) {
        console.error(`[Tauri IPC Error] Graceful Terminate ${symbol}:`, err);
        throw err;
      }
    } else {
      console.log(`[Browser Mock IPC] Graceful Terminate asset: ${symbol}`);
    }
  }

  /**
   * Ngắt khẩn cấp (IMMEDIATE TERMINATE)
   */
  static async immediateTerminateAsset(symbol: string): Promise<void> {
    if (isTauriEnvironment()) {
      try {
        await invoke('cmd_terminate_immediate', { asset: symbol });
      } catch (err) {
        console.error(`[Tauri IPC Error] Immediate Terminate ${symbol}:`, err);
        throw err;
      }
    } else {
      console.log(`[Browser Mock IPC] Immediate Terminate asset: ${symbol}`);
    }
  }

  /**
   * Dừng Khẩn Cấp Toàn Cục (EMERGENCY STOP ALL)
   */
  static async emergencyStopAll(): Promise<void> {
    if (isTauriEnvironment()) {
      try {
        await invoke('cmd_emergency_stop_all');
      } catch (err) {
        console.error('[Tauri IPC Error] Emergency Stop All:', err);
        throw err;
      }
    } else {
      console.log('[Browser Mock IPC] Emergency Stop All executed.');
    }
  }
}
