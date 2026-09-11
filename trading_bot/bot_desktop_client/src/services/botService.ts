// trading_bot/bot_desktop_client/src/services/botService.ts
import { invoke } from '@tauri-apps/api/core';
import { 
  AccountState, 
  AssetState, 
  SystemHealth, 
  RebalanceOption, 
  UnifiedExitAction 
} from '../types/bot';

/**
 * Kiểm tra xem app có đang chạy trong môi trường Desktop Tauri hay không
 */
export function isTauriEnvironment(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

/**
 * Danh sách Cặp tiền Mặc định cho bản Demo Local Desktop
 */
export const INITIAL_MOCK_ASSETS: AssetState[] = [
  {
    symbol: 'ETHUSDT',
    marketType: 'FUTURES 20x',
    status: 'INACTIVE',
    strategyName: 'Helios Strategy V3',
    fitnessReturnPct: 125.16,
    fitnessMaxDdPct: -12.34,
    initialBaseCapital: 60.0,
    realizedPnl: 25.40,
    allocatedCapital: 85.40,
    isSafeLiquidity: true,
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
    initialBaseCapital: 60.0,
    realizedPnl: 0.0,
    allocatedCapital: 60.0,
    isSafeLiquidity: true,
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
    initialBaseCapital: 60.0,
    realizedPnl: 48.20,
    allocatedCapital: 108.20,
    isSafeLiquidity: true,
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
  availableBalance: 9746.40,
  totalUnrealizedPnl: 0.00,
  activeAssetsCount: 0,
};

export const INITIAL_MOCK_HEALTH: SystemHealth = {
  apiWeightUsed: 45,
  apiWeightMax: 2400,
  wsConnected: true,
  latencyMs: 38,
  sqliteStoreActive: true,
};

/**
 * Service giao tiếp IPC chuẩn hóa giữa React Client & Rust Desktop Backend
 */
export class BotService {
  /**
   * Khởi động Bot cho một Cặp giao dịch (START_ASSET)
   */
  static async startAsset(symbol: string): Promise<void> {
    if (isTauriEnvironment()) {
      try {
        await invoke('cmd_start_asset', { symbol });
      } catch (err) {
        console.error(`[Tauri IPC Error] Start ${symbol}:`, err);
        throw err;
      }
    } else {
      console.log(`[Browser Mock IPC] Start asset: ${symbol}`);
    }
  }

  /**
   * Hủy tiến trình khởi động (CANCEL_START)
   */
  static async cancelStart(symbol: string): Promise<void> {
    if (isTauriEnvironment()) {
      try {
        await invoke('cmd_cancel_start', { symbol });
      } catch (err) {
        console.error(`[Tauri IPC Error] Cancel Start ${symbol}:`, err);
        throw err;
      }
    } else {
      console.log(`[Browser Mock IPC] Cancel Start asset: ${symbol}`);
    }
  }

  /**
   * Tạm dừng tìm entry (PAUSE_ASSET)
   */
  static async pauseAsset(symbol: string): Promise<void> {
    if (isTauriEnvironment()) {
      try {
        await invoke('cmd_pause_asset', { symbol });
      } catch (err) {
        console.error(`[Tauri IPC Error] Pause ${symbol}:`, err);
        throw err;
      }
    } else {
      console.log(`[Browser Mock IPC] Pause asset: ${symbol}`);
    }
  }

  /**
   * Tiếp tục giao dịch (RESUME_ASSET)
   */
  static async resumeAsset(symbol: string): Promise<void> {
    if (isTauriEnvironment()) {
      try {
        await invoke('cmd_resume_asset', { symbol });
      } catch (err) {
        console.error(`[Tauri IPC Error] Resume ${symbol}:`, err);
        throw err;
      }
    } else {
      console.log(`[Browser Mock IPC] Resume asset: ${symbol}`);
    }
  }

  /**
   * Dừng an toàn chờ Trailing SL chốt lệnh (GRACEFUL_STOP)
   */
  static async gracefulStop(symbol: string): Promise<void> {
    if (isTauriEnvironment()) {
      try {
        await invoke('cmd_graceful_stop', { symbol });
      } catch (err) {
        console.error(`[Tauri IPC Error] Graceful Stop ${symbol}:`, err);
        throw err;
      }
    } else {
      console.log(`[Browser Mock IPC] Graceful Stop asset: ${symbol}`);
    }
  }

  /**
   * Hủy lệnh dừng an toàn (CANCEL_GRACEFUL_STOP)
   */
  static async cancelGracefulStop(symbol: string): Promise<void> {
    if (isTauriEnvironment()) {
      try {
        await invoke('cmd_cancel_graceful_stop', { symbol });
      } catch (err) {
        console.error(`[Tauri IPC Error] Cancel Graceful Stop ${symbol}:`, err);
        throw err;
      }
    } else {
      console.log(`[Browser Mock IPC] Cancel Graceful Stop asset: ${symbol}`);
    }
  }

  /**
   * Ngắt khẩn cấp đóng Market ngay (FORCE_STOP_ASSET)
   */
  static async forceStopAsset(symbol: string): Promise<void> {
    if (isTauriEnvironment()) {
      try {
        await invoke('cmd_force_stop_asset', { symbol });
      } catch (err) {
        console.error(`[Tauri IPC Error] Force Stop ${symbol}:`, err);
        throw err;
      }
    } else {
      console.log(`[Browser Mock IPC] Force Stop asset: ${symbol}`);
    }
  }

  /**
   * Điều chuyển vốn an toàn (REBALANCE_CAPITAL)
   */
  static async rebalanceCapital(
    fromSymbol: string, 
    toSymbol: string, 
    amount: number, 
    option: RebalanceOption
  ): Promise<void> {
    if (isTauriEnvironment()) {
      try {
        await invoke('cmd_rebalance_capital', { 
          fromSymbol, 
          toSymbol, 
          amount, 
          option 
        });
      } catch (err) {
        console.error('[Tauri IPC Error] Rebalance Capital:', err);
        throw err;
      }
    } else {
      console.log(`[Browser Mock IPC] Rebalance ${amount}$ from ${fromSymbol} to ${toSymbol} with option ${option}`);
    }
  }

  /**
   * Dừng Khẩn Cấp Toàn Cục (EMERGENCY_STOP_ALL)
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

  /**
   * Thực hiện thoát ứng dụng tổng hợp (UNIFIED_EXIT)
   */
  static async unifiedExit(action: UnifiedExitAction): Promise<void> {
    if (isTauriEnvironment()) {
      try {
        await invoke('cmd_unified_exit', { action });
      } catch (err) {
        console.error('[Tauri IPC Error] Unified Exit:', err);
        throw err;
      }
    } else {
      console.log(`[Browser Mock IPC] Unified Exit action: ${action}`);
    }
  }
}
