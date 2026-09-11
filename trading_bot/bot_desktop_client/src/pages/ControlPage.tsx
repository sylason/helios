// trading_bot/bot_desktop_client/src/pages/ControlPage.tsx
import React, { useState } from 'react';
import { Container } from 'react-bootstrap';
import { AssetState } from '../types/bot';
import { WatchlistTable } from '../components/control/WatchlistTable';
import { AssetLogPanel } from '../components/control/AssetLogPanel';

interface ControlPageProps {
  assets: AssetState[];
  onStart: (symbol: string) => void;
  onCancelStart: (symbol: string) => void;
  onPause: (symbol: string) => void;
  onResume: (symbol: string) => void;
  onGracefulStop: (symbol: string) => void;
  onCancelGracefulStop: (symbol: string) => void;
  onForceStop: (symbol: string) => void;
  onOpenRebalance?: (symbol: string) => void;
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
  onOpenRebalance,
}) => {
  const [selectedSymbol, setSelectedSymbol] = useState<string>(
    assets.length > 0 ? assets[0].symbol : 'ETHUSDT'
  );

  const selectedAsset = assets.find((a) => a.symbol === selectedSymbol) || assets[0];

  return (
    <Container fluid className="p-4">
      {/* Table Component */}
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
        onOpenRebalance={onOpenRebalance}
      />

      {/* Asset Log Stream Component */}
      {selectedAsset && (
        <AssetLogPanel 
          symbol={selectedAsset.symbol} 
          logs={selectedAsset.recentLogs} 
        />
      )}
    </Container>
  );
};
