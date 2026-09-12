// trading_bot/bot_desktop_client/src/pages/ControlPage.tsx
import React, { useState } from 'react';
import { Container } from 'react-bootstrap';
import { AssetState, RebalanceOption } from '../types/bot';
import { WatchlistTable } from '../components/control/WatchlistTable';
import { AssetLogPanel } from '../components/control/AssetLogPanel';
import { RebalanceModal } from '../components/control/RebalanceModal';

interface ControlPageProps {
  assets: AssetState[];
  onStart: (symbol: string) => void;
  onCancelStart: (symbol: string) => void;
  onPause: (symbol: string) => void;
  onResume: (symbol: string) => void;
  onGracefulStop: (symbol: string) => void;
  onCancelGracefulStop: (symbol: string) => void;
  onForceStop: (symbol: string) => void;
  onConfirmRebalance: (
    fromSymbol: string,
    toSymbol: string,
    amount: number,
    option: RebalanceOption
  ) => Promise<void>;
}

export const ControlPage: React.FC<ControlPageProps> = ({
  assets,
  onStart,
  onCancelStart,
  onPause,
  onResume,
  onGracefulStop,
  onCancelGracefulStop,
  onForceStop,
  onConfirmRebalance,
}) => {
  const [selectedSymbol, setSelectedSymbol] = useState<string>(
    assets.length > 0 ? assets[0].symbol : 'ETHUSDT'
  );

  // State quản lý mở Modal Điều Chuyển Vốn
  const [rebalanceSourceSymbol, setRebalanceSourceSymbol] = useState<string | null>(null);

  const selectedAsset = assets.find((a) => a.symbol === selectedSymbol) || assets[0];
  const rebalanceSourceAsset = assets.find((a) => a.symbol === rebalanceSourceSymbol) || null;

  return (
    <Container fluid className="p-4">
      {/* 1. Bảng Quản Trị Cặp Tiền (Watchlist Table) */}
      <WatchlistTable
        assets={assets}
        selectedSymbol={selectedSymbol}
        onSelectAsset={(symbol) => setSelectedSymbol(symbol)}
        onStart={onStart}
        onCancelStart={onCancelStart}
        onPause={onPause}
        onResume={onResume}
        onGracefulStop={onGracefulStop}
        onCancelGracefulStop={onCancelGracefulStop}
        onForceStop={onForceStop}
        onOpenRebalance={(symbol) => setRebalanceSourceSymbol(symbol)}
      />

      {/* 2. Bảng Event Log Thu Nhỏ của Cặp Được Chọn */}
      {selectedAsset && (
        <AssetLogPanel 
          symbol={selectedAsset.symbol} 
          logs={selectedAsset.recentLogs} 
        />
      )}

      {/* 3. Hộp Thoại Điều Chuyển Vốn An Toàn (Rebalance Modal) */}
      <RebalanceModal
        show={rebalanceSourceSymbol !== null}
        onHide={() => setRebalanceSourceSymbol(null)}
        sourceAsset={rebalanceSourceAsset}
        allAssets={assets}
        onConfirmRebalance={onConfirmRebalance}
      />
    </Container>
  );
};
