// trading_bot/bot_desktop_client/src/types/bot.ts

/**
 * Máy trạng thái vận hành của từng Cặp giao dịch (Asset)
 * Khớp chuẩn 1:1 với FSM trong Rust Backend (EngineController)
 */
export type AssetStatus = 
  | 'INACTIVE' 
  | 'STARTING' 
  | 'RUNNING_ACTIVE' 
  | 'PAUSED' 
  | 'GRACEFUL_STOPPING' 
  | 'STOPPED' 
  | 'ERROR';

/**
 * Phân kỳ giai đoạn khi Asset đang ở trạng thái STARTING
 */
export type StartingPhase = 'SYNCING' | 'OPTIMIZING';

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
  startingPhase?: StartingPhase;
  strategyName: string;
  fitnessReturnPct: number;
  fitnessMaxDdPct: number;
  initialBaseCapital: number;
  realizedPnl: number;
  allocatedCapital: number;
  isSafeLiquidity: boolean;
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
  sqliteStoreActive?: boolean;
}

/**
 * Định danh trang ứng dụng (State-based Navigation Routing)
 */
export type PageId = 'control' | 'dashboard';

/**
 * Chế độ lọc nhanh trên thanh công cụ Watchlist
 */
export type FastFilterMode = 'ALL' | 'ACTIVE' | 'PAUSED' | 'INACTIVE' | 'SAFE_LIQUIDITY';

/**
 * Các hành động lựa chọn khi thực hiện Bảng Thoát Tổng Hợp (Unified Exit Modal)
 */
export type UnifiedExitAction = 'FLAT_ALL' | 'LEAVE_CLOUD_SL' | 'CANCEL';

/**
 * Tùy chọn khi điều chuyển vốn (Capital Rebalance Modal)
 */
export type RebalanceOption = 'APPLY_NEXT_TRADE' | 'CLOSE_IMMEDIATELY';

/**
 * Cấu trúc thông báo Toast nổi (Floating Toast Notification)
 */
export interface ToastNotification {
  id: string;
  title: string;
  message: string;
  variant: 'danger' | 'warning' | 'success' | 'info';
  timestamp: number;
  symbol?: string;
}
