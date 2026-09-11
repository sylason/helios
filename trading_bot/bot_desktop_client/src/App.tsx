// trading_bot/bot_desktop_client/src/App.tsx
import { useState, useMemo } from 'react';
import { PageId, AssetState, AccountState, SystemHealth } from './types/bot';
import { 
  INITIAL_MOCK_ASSETS, 
  INITIAL_MOCK_ACCOUNT, 
  INITIAL_MOCK_HEALTH, 
  BotService 
} from './services/botService';
import { Topbar } from './components/layout/Topbar';
import { Sidebar } from './components/layout/Sidebar';
import { ControlPage } from './pages/ControlPage';
import { DashboardPage } from './pages/DashboardPage';

function App() {
  const [activePage, setActivePage] = useState<PageId>('control');
  const [assets, setAssets] = useState<AssetState[]>(INITIAL_MOCK_ASSETS);
  const [account, setAccount] = useState<AccountState>(INITIAL_MOCK_ACCOUNT);
  const [health] = useState<SystemHealth>(INITIAL_MOCK_HEALTH);

  // Tính số lượng Asset đang Active (RUNNING_ACTIVE, STARTING, GRACEFUL_STOPPING, PAUSED)
  const activeCount = useMemo(() => {
    return assets.filter(a => 
      a.status === 'RUNNING_ACTIVE' || 
      a.status === 'STARTING' || 
      a.status === 'GRACEFUL_STOPPING' || 
      a.status === 'PAUSED'
    ).length;
  }, [assets]);

  // Cập nhật state Account khi activeCount thay đổi
  const currentAccount = useMemo(() => {
    return {
      ...account,
      activeAssetsCount: activeCount,
    };
  }, [account, activeCount]);

  /**
   * 1. Kích hoạt Bot cho một Cặp tiền (START_ASSET: STARTING Sync -> JIT GA Optimize -> RUNNING_ACTIVE)
   */
  const handleStart = async (symbol: string) => {
    setAssets(prev => prev.map(a => {
      if (a.symbol === symbol) {
        return {
          ...a,
          status: 'STARTING',
          startingPhase: 'SYNCING',
          recentLogs: [...a.recentLogs, `[${new Date().toLocaleTimeString()}] 📥 Starting 33-Day Kline Sync from Binance REST...`],
        };
      }
      return a;
    }));

    await BotService.startAsset(symbol);

    // Giả lập sau 1.5s chuyển sang JIT GA
    setTimeout(() => {
      setAssets(prev => prev.map(a => {
        if (a.symbol === symbol) {
          return {
            ...a,
            status: 'STARTING',
            startingPhase: 'OPTIMIZING',
            recentLogs: [...a.recentLogs, `[${new Date().toLocaleTimeString()}] 🧬 Prospector GA Engine building Champion DNA (JIT)...`],
          };
        }
        return a;
      }));

      // Giả lập sau 1.5s nữa chuyển sang RUNNING_ACTIVE
      setTimeout(() => {
        setAssets(prev => prev.map(a => {
          if (a.symbol === symbol) {
            const isLong = symbol === 'ETHUSDT' || symbol === 'SOLUSDT';
            return {
              ...a,
              status: 'RUNNING_ACTIVE',
              startingPhase: undefined,
              position: {
                side: isLong ? 'LONG' : 'SHORT',
                qty: symbol === 'ETHUSDT' ? 1.5 : (symbol === 'BTCUSDT' ? 0.05 : 10.0),
                entryPrice: symbol === 'ETHUSDT' ? 1905.40 : (symbol === 'BTCUSDT' ? 64200.0 : 135.50),
                unrealizedPnl: 45.20,
                unrealizedPnlPct: 2.37,
                trailingSlPrice: symbol === 'ETHUSDT' ? 1929.54 : undefined,
              },
              recentLogs: [
                ...a.recentLogs, 
                `[${new Date().toLocaleTimeString()}] 🚀 Champion DNA loaded. RUNNING_ACTIVE on Binance Futures Testnet.`,
                `[${new Date().toLocaleTimeString()}] 🛡️ Mark Price Trailing SL order initialized.`,
              ],
            };
          }
          return a;
        }));

        setAccount(prev => ({
          ...prev,
          totalUnrealizedPnl: prev.totalUnrealizedPnl + 45.20,
        }));
      }, 1500);
    }, 1500);
  };

  /**
   * 2. Hủy tiến trình khởi động (CANCEL_START)
   */
  const handleCancelStart = async (symbol: string) => {
    setAssets(prev => prev.map(a => {
      if (a.symbol === symbol) {
        return {
          ...a,
          status: 'INACTIVE',
          startingPhase: undefined,
          recentLogs: [...a.recentLogs, `[${new Date().toLocaleTimeString()}] ✕ Cancel start requested. State returned to INACTIVE.`],
        };
      }
      return a;
    }));

    await BotService.cancelStart(symbol);
  };

  /**
   * 3. Tạm dừng tìm entry (PAUSE_ASSET)
   */
  const handlePause = async (symbol: string) => {
    setAssets(prev => prev.map(a => {
      if (a.symbol === symbol) {
        return {
          ...a,
          status: 'PAUSED',
          recentLogs: [...a.recentLogs, `[${new Date().toLocaleTimeString()}] ⏸️ Bot paused. Holding existing positions with Trailing SL.`],
        };
      }
      return a;
    }));

    await BotService.pauseAsset(symbol);
  };

  /**
   * 4. Tiếp tục giao dịch (RESUME_ASSET)
   */
  const handleResume = async (symbol: string) => {
    setAssets(prev => prev.map(a => {
      if (a.symbol === symbol) {
        return {
          ...a,
          status: 'RUNNING_ACTIVE',
          recentLogs: [...a.recentLogs, `[${new Date().toLocaleTimeString()}] ▶️ Bot resumed. Scanning live candles for entries.`],
        };
      }
      return a;
    }));

    await BotService.resumeAsset(symbol);
  };

  /**
   * 5. Dừng an toàn (GRACEFUL_STOP)
   */
  const handleGracefulStop = async (symbol: string) => {
    setAssets(prev => prev.map(a => {
      if (a.symbol === symbol) {
        return {
          ...a,
          status: 'GRACEFUL_STOPPING',
          recentLogs: [...a.recentLogs, `[${new Date().toLocaleTimeString()}] 🟡 Graceful stop requested. Waiting for Trailing SL execution...`],
        };
      }
      return a;
    }));

    await BotService.gracefulStop(symbol);

    // Giả lập sau 3s đóng xong vị thế
    setTimeout(() => {
      setAssets(prev => prev.map(a => {
        if (a.symbol === symbol) {
          return {
            ...a,
            status: 'INACTIVE',
            position: { side: 'FLAT', qty: 0, entryPrice: 0, unrealizedPnl: 0, unrealizedPnlPct: 0 },
            recentLogs: [...a.recentLogs, `[${new Date().toLocaleTimeString()}] 🧹 Trailing SL filled. Position closed gracefully. Asset INACTIVE.`],
          };
        }
        return a;
      }));
    }, 3000);
  };

  /**
   * 6. Hủy dừng an toàn (CANCEL_GRACEFUL_STOP)
   */
  const handleCancelGracefulStop = async (symbol: string) => {
    setAssets(prev => prev.map(a => {
      if (a.symbol === symbol) {
        return {
          ...a,
          status: 'RUNNING_ACTIVE',
          recentLogs: [...a.recentLogs, `[${new Date().toLocaleTimeString()}] ↩️ Graceful stop cancelled. Resumed active trading.`],
        };
      }
      return a;
    }));

    await BotService.cancelGracefulStop(symbol);
  };

  /**
   * 7. Ngắt khẩn cấp đóng Market ngay (FORCE_STOP_ASSET)
   */
  const handleForceStop = async (symbol: string) => {
    setAssets(prev => prev.map(a => {
      if (a.symbol === symbol) {
        return {
          ...a,
          status: 'INACTIVE',
          position: { side: 'FLAT', qty: 0, entryPrice: 0, unrealizedPnl: 0, unrealizedPnlPct: 0 },
          recentLogs: [...a.recentLogs, `[${new Date().toLocaleTimeString()}] 🔴 Immediate force stop executed. Algo orders cancelled. Position reset.`],
        };
      }
      return a;
    }));

    await BotService.forceStopAsset(symbol);
  };

  /**
   * 8. Mở Modal Điều Chuyển Vốn
   */
  const handleOpenRebalance = (symbol: string) => {
    console.log(`[Rebalance Trigger] Open rebalance modal for ${symbol}`);
  };

  /**
   * 9. Dừng khẩn cấp Toàn cục (EMERGENCY_STOP_ALL)
   */
  const handleEmergencyStopAll = async () => {
    setAssets(prev => prev.map(a => ({
      ...a,
      status: 'INACTIVE',
      position: { side: 'FLAT', qty: 0, entryPrice: 0, unrealizedPnl: 0, unrealizedPnlPct: 0 },
      recentLogs: [...a.recentLogs, `[${new Date().toLocaleTimeString()}] 🚨 GLOBAL EMERGENCY STOP EXECUTED. All bots stopped.`],
    })));

    setAccount(prev => ({
      ...prev,
      totalUnrealizedPnl: 0,
    }));

    await BotService.emergencyStopAll();
  };

  return (
    <div className="bg-dark min-vh-100 text-light d-flex flex-column">
      {/* Topbar Component */}
      <Topbar wsConnected={health.wsConnected} />

      {/* Main Content Layout (Sidebar + Active Page) */}
      <div className="d-flex flex-grow-1">
        <Sidebar 
          activePage={activePage} 
          onSelectPage={(page) => setActivePage(page)} 
        />

        <main className="flex-grow-1 bg-dark overflow-auto">
          {activePage === 'control' && (
            <ControlPage 
              assets={assets}
              onStart={handleStart}
              onCancelStart={handleCancelStart}
              onPause={handlePause}
              onResume={handleResume}
              onGracefulStop={handleGracefulStop}
              onCancelGracefulStop={handleCancelGracefulStop}
              onForceStop={handleForceStop}
              onOpenRebalance={handleOpenRebalance}
            />
          )}

          {activePage === 'dashboard' && (
            <DashboardPage 
              account={currentAccount}
              health={health}
              totalAssetsCount={assets.length}
              onEmergencyStopAll={handleEmergencyStopAll}
            />
          )}
        </main>
      </div>
    </div>
  );
}

export default App;
