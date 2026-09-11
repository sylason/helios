// trading_bot/bot_desktop_client/src/components/control/WatchlistTable.tsx
import React, { useState, useMemo } from 'react';
import { Table, Button, Badge, Card, Form, InputGroup, ButtonGroup } from 'react-bootstrap';
import { AssetState, FastFilterMode } from '../../types/bot';
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
  onStart: (symbol: string) => void;
  onCancelStart: (symbol: string) => void;
  onPause: (symbol: string) => void;
  onResume: (symbol: string) => void;
  onGracefulStop: (symbol: string) => void;
  onCancelGracefulStop: (symbol: string) => void;
  onForceStop: (symbol: string) => void;
  onOpenRebalance?: (symbol: string) => void;
}

export const WatchlistTable: React.FC<WatchlistTableProps> = ({
  assets,
  selectedSymbol,
  onSelectAsset,
  onStart,
  onCancelStart,
  onPause,
  onResume,
  onGracefulStop,
  onCancelGracefulStop,
  onForceStop,
  onOpenRebalance,
}) => {
  const [searchQuery, setSearchQuery] = useState<string>('');
  const [filterMode, setFilterMode] = useState<FastFilterMode>('ALL');

  /**
   * Lọc danh sách Asset theo từ khóa tìm kiếm và chế độ Tab
   */
  const filteredAssets = useMemo(() => {
    return assets.filter((asset) => {
      // 1. Lọc theo từ khóa tìm kiếm
      const query = searchQuery.toLowerCase().trim();
      const matchesSearch = query === '' || 
        asset.symbol.toLowerCase().includes(query) || 
        asset.marketType.toLowerCase().includes(query);
      
      if (!matchesSearch) return false;

      // 2. Lọc theo chế độ Tab
      switch (filterMode) {
        case 'ACTIVE':
          return asset.status === 'RUNNING_ACTIVE' || 
                 asset.status === 'STARTING' || 
                 asset.status === 'GRACEFUL_STOPPING';
        case 'PAUSED':
          return asset.status === 'PAUSED';
        case 'INACTIVE':
          return asset.status === 'INACTIVE' || asset.status === 'STOPPED';
        case 'SAFE_LIQUIDITY':
          return asset.isSafeLiquidity;
        case 'ALL':
        default:
          return true;
      }
    });
  }, [assets, searchQuery, filterMode]);

  return (
    <Card bg="dark" border="secondary" className="shadow-sm mb-4">
      {/* Header với Tiêu đề & Công cụ Tìm kiếm Inline + Filter Tabs */}
      <Card.Header className="bg-black border-secondary py-3">
        <div className="d-flex flex-column flex-md-row justify-content-between align-items-md-center gap-3">
          {/* Tiêu đề & Tổng số lượng */}
          <div className="d-flex align-items-center gap-2">
            <h5 className="mb-0 fw-bold text-info">Active Watchlist & Bot Control</h5>
            <Badge bg="secondary" className="ms-1">{filteredAssets.length}/{assets.length} Cặp</Badge>
          </div>

          {/* Ô Tìm Kiếm Symbol Trực Tiếp (Inline Search) */}
          <div style={{ maxWidth: '320px', width: '100%' }}>
            <InputGroup size="sm">
              <InputGroup.Text className="bg-dark border-secondary text-secondary">
                🔍
              </InputGroup.Text>
              <Form.Control
                type="text"
                placeholder="Tìm cặp tiền (vd: ETH, BTC)..."
                className="bg-dark text-light border-secondary shadow-none"
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
              />
              {searchQuery && (
                <Button 
                  variant="outline-secondary" 
                  onClick={() => setSearchQuery('')}
                  title="Xóa tìm kiếm"
                >
                  ✕
                </Button>
              )}
            </InputGroup>
          </div>
        </div>

        {/* Thanh Tab Lọc Nhanh (Fast Filter Tabs) */}
        <div className="mt-3 d-flex flex-wrap gap-2">
          <ButtonGroup size="sm">
            <Button
              variant={filterMode === 'ALL' ? 'info' : 'outline-secondary'}
              onClick={() => setFilterMode('ALL')}
            >
              Tất Cả ({assets.length})
            </Button>
            <Button
              variant={filterMode === 'ACTIVE' ? 'success' : 'outline-secondary'}
              onClick={() => setFilterMode('ACTIVE')}
            >
              🟢 Đang Chạy
            </Button>
            <Button
              variant={filterMode === 'PAUSED' ? 'warning' : 'outline-secondary'}
              onClick={() => setFilterMode('PAUSED')}
            >
              ⏸️ Tạm Dừng
            </Button>
            <Button
              variant={filterMode === 'INACTIVE' ? 'secondary' : 'outline-secondary'}
              onClick={() => setFilterMode('INACTIVE')}
            >
              ⚪ Đang Tắt
            </Button>
            <Button
              variant={filterMode === 'SAFE_LIQUIDITY' ? 'primary' : 'outline-secondary'}
              onClick={() => setFilterMode('SAFE_LIQUIDITY')}
            >
              💧 Thanh Khoản An Toàn
            </Button>
          </ButtonGroup>
        </div>
      </Card.Header>

      {/* Bảng Danh Sách Cặp Giao Dịch */}
      <Card.Body className="p-0">
        <Table responsive variant="dark" hover className="mb-0 align-middle text-center border-0">
          <thead className="text-secondary text-uppercase small border-bottom border-secondary bg-black">
            <tr>
              <th className="py-3 text-start ps-4">Cặp Tiền & Thanh Khoản</th>
              <th>Trạng Thái FSM</th>
              <th>Vốn Cấp (Lãi Kép)</th>
              <th>Chiến Lược & DNA</th>
              <th>Vị Thế Live</th>
              <th>Trailing SL Target</th>
              <th style={{ minWidth: '180px' }}>Hành Động Chính</th>
              <th style={{ minWidth: '160px' }}>Thoát An Toàn</th>
            </tr>
          </thead>
          <tbody>
            {filteredAssets.length === 0 ? (
              <tr>
                <td colSpan={8} className="py-5 text-secondary">
                  <div className="fs-5 mb-2">Không tìm thấy cặp tiền phù hợp</div>
                  <small>Thử nhập từ khóa khác hoặc chuyển sang tab "Tất Cả"</small>
                </td>
              </tr>
            ) : (
              filteredAssets.map((asset) => {
                const isSelected = selectedSymbol === asset.symbol;
                const isStarting = asset.status === 'STARTING';
                const isRunning = asset.status === 'RUNNING_ACTIVE';
                const isPaused = asset.status === 'PAUSED';
                const isGraceful = asset.status === 'GRACEFUL_STOPPING';
                const isInactive = asset.status === 'INACTIVE' || asset.status === 'STOPPED';
                const isError = asset.status === 'ERROR';

                return (
                  <tr 
                    key={asset.symbol}
                    onClick={() => onSelectAsset(asset.symbol)}
                    className={isSelected ? 'table-active border-start border-3 border-info' : ''}
                    style={{ cursor: 'pointer' }}
                  >
                    {/* 1. Symbol & Huy Hiệu Thanh Khoản An Toàn */}
                    <td className="text-start ps-4">
                      <div className="d-flex align-items-center gap-2">
                        <span className="fw-bold fs-6 text-light">{asset.symbol}</span>
                        {asset.isSafeLiquidity && (
                          <Badge 
                            bg="primary" 
                            className="bg-opacity-25 text-info border border-info border-opacity-50 small"
                            title="Đạt chuẩn khối lượng 24h > $50M & spread thấp"
                          >
                            💧 Safe
                          </Badge>
                        )}
                      </div>
                      <small className="text-secondary opacity-75">{asset.marketType}</small>
                    </td>

                    {/* 2. Trạng Thái FSM Badges */}
                    <td>
                      <Badge 
                        bg={getStatusBadgeVariant(asset.status)} 
                        className={`px-3 py-2 text-uppercase ${isStarting ? 'placeholder-wave' : ''}`}
                      >
                        {getStatusLabel(asset.status, asset.startingPhase)}
                      </Badge>
                    </td>

                    {/* 3. Vốn Phân Bổ Lãi Kép & Nút Rebalance */}
                    <td>
                      <div className="fw-bold text-light">
                        {formatCurrency(asset.allocatedCapital || asset.initialBaseCapital)}
                      </div>
                      <div className="d-flex align-items-center justify-content-center gap-1 small text-secondary">
                        <span>Gốc: {formatCurrency(asset.initialBaseCapital)}</span>
                        {asset.realizedPnl !== 0 && (
                          <span className={asset.realizedPnl > 0 ? 'text-success' : 'text-danger'}>
                            ({asset.realizedPnl > 0 ? '+' : ''}{formatCurrency(asset.realizedPnl)})
                          </span>
                        )}
                        {onOpenRebalance && (
                          <button
                            type="button"
                            className="btn btn-link btn-sm p-0 ms-1 text-info text-decoration-none"
                            title="Điều chuyển vốn an toàn"
                            onClick={(e) => {
                              e.stopPropagation();
                              onOpenRebalance(asset.symbol);
                            }}
                          >
                            ⚙️
                          </button>
                        )}
                      </div>
                    </td>

                    {/* 4. Champion DNA Metrics */}
                    <td>
                      <div className="fw-semibold text-info small">{asset.strategyName}</div>
                      <small className="text-light">
                        Lãi: <span className="text-success fw-bold">+{asset.fitnessReturnPct}%</span> | 
                        DD: <span className="text-danger fw-bold">{asset.fitnessMaxDdPct}%</span>
                      </small>
                    </td>

                    {/* 5. Vị Thế Thời Gian Thực */}
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

                    {/* 6. Mức Trailing SL Target (Mark Price) */}
                    <td>
                      {asset.position.trailingSlPrice ? (
                        <span className="fw-bold text-warning border border-warning px-2 py-1 rounded small">
                          🛡️ {asset.position.trailingSlPrice}
                        </span>
                      ) : (
                        <span className="text-secondary small opacity-50">None</span>
                      )}
                    </td>

                    {/* 7. Bộ Nút Hành Động Chính (Context-Aware) */}
                    <td onClick={(e) => e.stopPropagation()}>
                      {isInactive && (
                        <Button 
                          variant="success" 
                          size="sm"
                          className="w-100 py-1 fw-bold text-uppercase shadow-sm"
                          onClick={() => onStart(asset.symbol)}
                        >
                          ▶ BẬT BOT
                        </Button>
                      )}

                      {isStarting && (
                        <Button 
                          variant="outline-danger" 
                          size="sm"
                          className="w-100 py-1 fw-bold text-uppercase"
                          onClick={() => onCancelStart(asset.symbol)}
                        >
                          ✕ HỦY BẬT
                        </Button>
                      )}

                      {isRunning && (
                        <Button 
                          variant="warning" 
                          size="sm"
                          className="w-100 py-1 fw-bold text-uppercase shadow-sm"
                          onClick={() => onPause(asset.symbol)}
                        >
                          ⏸️ TẠM DỪNG
                        </Button>
                      )}

                      {isPaused && (
                        <Button 
                          variant="info" 
                          size="sm"
                          className="w-100 py-1 fw-bold text-uppercase shadow-sm"
                          onClick={() => onResume(asset.symbol)}
                        >
                          ▶️ TIẾP TỤC
                        </Button>
                      )}

                      {isGraceful && (
                        <Button 
                          variant="outline-info" 
                          size="sm"
                          className="w-100 py-1 fw-semibold text-uppercase"
                          onClick={() => onCancelGracefulStop(asset.symbol)}
                        >
                          ↩️ HỦY DỪNG
                        </Button>
                      )}

                      {isError && (
                        <Button 
                          variant="outline-warning" 
                          size="sm"
                          className="w-100 py-1 fw-bold text-uppercase"
                          onClick={() => onStart(asset.symbol)}
                        >
                          🔄 THỬ LẠI
                        </Button>
                      )}
                    </td>

                    {/* 8. Bộ Nút Thoát An Toàn / Dừng Khẩn Cấp */}
                    <td onClick={(e) => e.stopPropagation()}>
                      {(isRunning || isPaused) && (
                        <div className="d-flex gap-1">
                          <Button 
                            variant="outline-warning" 
                            size="sm"
                            className="w-50 py-1 fw-semibold text-uppercase small"
                            title="Không vào lệnh mới, chờ Trailing SL chốt"
                            onClick={() => onGracefulStop(asset.symbol)}
                          >
                            🟡 DỪNG ÊM
                          </Button>
                          <Button 
                            variant="danger" 
                            size="sm"
                            className="w-50 py-1 fw-bold text-uppercase small shadow-sm"
                            title="Đóng Market vị thế ngay tức thì"
                            onClick={() => onForceStop(asset.symbol)}
                          >
                            🛑 DỪNG GẤP
                          </Button>
                        </div>
                      )}

                      {isGraceful && (
                        <Button 
                          variant="danger" 
                          size="sm"
                          className="w-100 py-1 fw-bold text-uppercase shadow-sm"
                          onClick={() => onForceStop(asset.symbol)}
                        >
                          ⚡ ĐÓNG MARKET NGAY
                        </Button>
                      )}

                      {(isInactive || isStarting || isError) && (
                        <span className="text-secondary small opacity-50">Sẵn sàng</span>
                      )}
                    </td>
                  </tr>
                );
              })
            )}
          </tbody>
        </Table>
      </Card.Body>
    </Card>
  );
};
