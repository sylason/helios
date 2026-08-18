// trading_bot/bot_desktop_client/src/components/control/WatchlistTable.tsx
import React from 'react';
import { Table, Button, Badge, Card } from 'react-bootstrap';
import { AssetState } from '../../types/bot';
import { 
  formatCurrency, 
  formatPercent, 
  getStatusBadgeVariant, 
  getStatusLabel, 
  getPositionBadgeVariant 
} from '../../utils/formatters';

interface WatchlistTableProps {
  assets: AssetState[];
  selectedSymbol: string;
  onSelectAsset: (symbol: string) => void;
  onActivate: (symbol: string) => void;
  onGracefulTerminate: (symbol: string) => void;
  onImmediateTerminate: (symbol: string) => void;
}

export const WatchlistTable: React.FC<WatchlistTableProps> = ({
  assets,
  selectedSymbol,
  onSelectAsset,
  onActivate,
  onGracefulTerminate,
  onImmediateTerminate,
}) => {
  return (
    <Card bg="dark" border="secondary" className="shadow-sm">
      <Card.Header className="bg-black text-info border-secondary py-3 d-flex justify-content-between align-items-center">
        <div className="d-flex align-items-center gap-2">
          <h5 className="mb-0 fw-bold">Active Watchlist & Bot Control</h5>
          <Badge bg="secondary" className="ms-2">{assets.length} Assets</Badge>
        </div>
        <small className="text-secondary">Click row to inspect live event logs</small>
      </Card.Header>

      <Card.Body className="p-0">
        <Table responsive variant="dark" hover className="mb-0 align-middle text-center border-0">
          <thead className="text-secondary text-uppercase small border-bottom border-secondary bg-black">
            <tr>
              <th className="py-3">Asset Symbol</th>
              <th>Status</th>
              <th>Champion DNA</th>
              <th>Live Position</th>
              <th>Trailing SL</th>
              <th style={{ width: '220px' }}>Main Action</th>
              <th style={{ width: '180px' }}>Safe Exit</th>
            </tr>
          </thead>
          <tbody>
            {assets.map((asset) => {
              const isSelected = selectedSymbol === asset.symbol;
              const isBusy = asset.status === 'SYNCING_DATA' || asset.status === 'GA_OPTIMIZING';
              const isActive = asset.status === 'READY_TRADING' || isBusy || asset.status === 'GRACEFUL_EXIT';

              return (
                <tr 
                  key={asset.symbol}
                  onClick={() => onSelectAsset(asset.symbol)}
                  className={isSelected ? 'table-active border-start border-3 border-info' : ''}
                  style={{ cursor: 'pointer' }}
                >
                  {/* Asset Symbol & Market */}
                  <td className="text-start ps-4">
                    <div className="fw-bold fs-6 text-light">{asset.symbol}</div>
                    <small className="text-secondary opacity-75">{asset.marketType}</small>
                  </td>

                  {/* Operational Status Badge */}
                  <td>
                    <Badge 
                      bg={getStatusBadgeVariant(asset.status)} 
                      className={`px-3 py-2 text-uppercase ${isBusy ? 'placeholder-wave' : ''}`}
                    >
                      {getStatusLabel(asset.status)}
                    </Badge>
                  </td>

                  {/* Champion DNA Metrics */}
                  <td>
                    <div className="fw-semibold text-info">{asset.strategyName}</div>
                    <small className="text-light">
                      Return: <span className="text-success fw-bold">+{asset.fitnessReturnPct}%</span> | 
                      DD: <span className="text-danger fw-bold">{asset.fitnessMaxDdPct}%</span>
                    </small>
                  </td>

                  {/* Live Position State */}
                  <td>
                    {asset.position.side === 'FLAT' ? (
                      <Badge bg="secondary" className="px-3 py-1 opacity-50">FLAT</Badge>
                    ) : (
                      <div>
                        <div className="d-flex justify-content-center align-items-center gap-2">
                          <Badge bg={getPositionBadgeVariant(asset.position.side)}>
                            {asset.position.side} {asset.position.qty}
                          </Badge>
                          <span className="small text-secondary">@{asset.position.entryPrice}</span>
                        </div>
                        <small className={asset.position.unrealizedPnl >= 0 ? 'text-success fw-bold' : 'text-danger fw-bold'}>
                          uPnL: {formatCurrency(asset.position.unrealizedPnl)} ({formatPercent(asset.position.unrealizedPnlPct)})
                        </small>
                      </div>
                    )}
                  </td>

                  {/* Target Trailing SL */}
                  <td>
                    {asset.position.trailingSlPrice ? (
                      <span className="fw-bold text-warning border border-warning px-2 py-1 rounded small">
                        🛡️ {asset.position.trailingSlPrice}
                      </span>
                    ) : (
                      <span className="text-secondary small opacity-50">None</span>
                    )}
                  </td>

                  {/* Main Action Button (ACTIVATE / STOP IMMEDIATELY) */}
                  <td onClick={(e) => e.stopPropagation()}>
                    {!isActive ? (
                      <Button 
                        variant="success" 
                        className="w-100 py-2 fw-bold text-uppercase shadow-sm"
                        onClick={() => onActivate(asset.symbol)}
                      >
                        🟢 ACTIVATE BOT
                      </Button>
                    ) : (
                      <Button 
                        variant="danger" 
                        className="w-100 py-2 fw-bold text-uppercase shadow-sm"
                        onClick={() => onImmediateTerminate(asset.symbol)}
                      >
                        🔴 STOP IMMEDIATELY
                      </Button>
                    )}
                  </td>

                  {/* Safe Exit Button (GRACEFUL STOP) */}
                  <td onClick={(e) => e.stopPropagation()}>
                    <Button 
                      variant="outline-warning" 
                      className="w-100 py-2 fw-semibold text-uppercase"
                      onClick={() => onGracefulTerminate(asset.symbol)}
                      disabled={!isActive || asset.status === 'GRACEFUL_EXIT'}
                    >
                      🟡 GRACEFUL EXIT
                    </Button>
                  </td>
                </tr>
              );
            })}
          </tbody>
        </Table>
      </Card.Body>
    </Card>
  );
};
