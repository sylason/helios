// trading_bot/bot_desktop_client/src/types/bot.ts

/**
 * Trạng thái vận hành của từng Cặp giao dịch (Asset)
 */
export type AssetStatus = 
  | 'INACTIVE' 
  | 'SYNCING_DATA' 
  | 'GA_OPTIMIZING' 
  | 'READY_TRADING' 
  | 'GRACEFUL_EXIT' 
  | 'ERROR';

/**
 * Phía vị thế: LONG, SHORT, hoặc FLAT (Không có vị thế)
 */
export type PositionSide = 'LONG' | 'SHORT' | 'FLAT';

/**
 * Trạng thái Vị thế thời gian thực (Live Position)
 */
export interface LivePosition {
  side: PositionSide;
  qty: number;
  entryPrice: number;
  unrealizedPnl: number;
  unrealizedPnlPct: number;
  trailingSlPrice?: number;
}

/**
 * Trạng thái đầy đủ của một Asset được quản lý bởi Bot
 */
export interface AssetState {
  symbol: string;
  marketType: string;
  status: AssetStatus;
  strategyName: string;
  fitnessReturnPct: number;
  fitnessMaxDdPct: number;
  position: LivePosition;
  recentLogs: string[];
}

/**
 * Trạng thái Tài khoản Tổng quan (Account Overview)
 */
export interface AccountState {
  walletBalance: number;
  availableBalance: number;
  totalUnrealizedPnl: number;
  activeAssetsCount: number;
}

/**
 * Trạng thái Sức khỏe Hệ thống & Kết nối (System Health)
 */
export interface SystemHealth {
  apiWeightUsed: number;
  apiWeightMax: number;
  wsConnected: boolean;
  latencyMs: number;
}

/**
 * Định danh trang ứng dụng (State-based Navigation Routing)
 */
export type PageId = 'control' | 'dashboard';
