// trading_bot/bot_desktop_client/src/components/layout/UnifiedExitModal.tsx
import React, { useState } from 'react';
import { Modal, Button, Table, Badge, Alert } from 'react-bootstrap';
import { AssetState, UnifiedExitAction } from '../../types/bot';
import { 
  formatCurrency, 
  formatPercent, 
  getPositionBadgeVariant 
} from '../../utils/formatters';

interface UnifiedExitModalProps {
  show: boolean;
  onHide: () => void;
  openPositions: AssetState[];
  onExecuteExit: (action: UnifiedExitAction) => Promise<void>;
}

export const UnifiedExitModal: React.FC<UnifiedExitModalProps> = ({
  show,
  onHide,
  openPositions,
  onExecuteExit,
}) => {
  const [isProcessing, setIsProcessing] = useState<boolean>(false);
  const [selectedAction, setSelectedAction] = useState<UnifiedExitAction | null>(null);

  const handleAction = async (action: UnifiedExitAction) => {
    if (action === 'CANCEL') {
      onHide();
      return;
    }

    setIsProcessing(true);
    setSelectedAction(action);
    try {
      await onExecuteExit(action);
      onHide();
    } catch (err) {
      console.error('Lỗi khi thực thi thoát ứng dụng:', err);
    } finally {
      setIsProcessing(false);
      setSelectedAction(null);
    }
  };

  return (
    <Modal 
      show={show} 
      onHide={onHide} 
      centered 
      size="lg"
      backdrop="static" 
      contentClassName="bg-dark text-light border border-secondary shadow-lg"
    >
      <Modal.Header className="border-secondary bg-black">
        <Modal.Title className="fs-5 fw-bold text-danger d-flex align-items-center gap-2">
          <span>⚠️ CẢNH BÁO THOÁT ỨNG DỤNG - DANH SÁCH VỊ THẾ ĐANG MỞ</span>
        </Modal.Title>
      </Modal.Header>

      <Modal.Body className="py-3">
        <Alert variant="danger" className="bg-danger bg-opacity-10 border-danger text-light small py-2 px-3 mb-3">
          Phần mềm phát hiện bạn đang có <strong>{openPositions.length} vị thế giao dịch đang mở</strong> trên sàn Binance Futures. 
          Vui lòng chọn phương án xử lý vị thế trước khi đóng phần mềm:
        </Alert>

        {/* Bảng Danh Sách Vị Thế 6 Cột */}
        <div className="border border-secondary rounded overflow-hidden mb-3">
          <Table responsive variant="dark" hover className="mb-0 align-middle text-center small">
            <thead className="text-secondary text-uppercase bg-black border-bottom border-secondary">
              <tr>
                <th className="py-2 text-start ps-3">Symbol</th>
                <th>Phía</th>
                <th>Khối Lượng</th>
                <th>Giá Khớp (Entry)</th>
                <th>Live uPnL</th>
                <th>Binance Trailing SL</th>
              </tr>
            </thead>
            <tbody>
              {openPositions.map((asset) => {
                const isProfit = asset.position.unrealizedPnl >= 0;
                return (
                  <tr key={asset.symbol}>
                    <td className="text-start ps-3 fw-bold text-light">{asset.symbol}</td>
                    <td>
                      <Badge bg={getPositionBadgeVariant(asset.position.side)}>
                        {asset.position.side}
                      </Badge>
                    </td>
                    <td>{asset.position.qty}</td>
                    <td>${asset.position.entryPrice.toLocaleString()}</td>
                    <td className={isProfit ? 'text-success fw-bold' : 'text-danger fw-bold'}>
                      {formatCurrency(asset.position.unrealizedPnl)} ({formatPercent(asset.position.unrealizedPnlPct)})
                    </td>
                    <td>
                      {asset.position.trailingSlPrice ? (
                        <span className="text-warning fw-bold border border-warning px-2 py-0 rounded small">
                          🛡️ ${asset.position.trailingSlPrice.toLocaleString()}
                        </span>
                      ) : (
                        <span className="text-secondary opacity-50">Không có</span>
                      )}
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </Table>
        </div>
      </Modal.Body>

      <Modal.Footer className="border-secondary bg-black d-flex flex-column flex-sm-row justify-content-between gap-2">
        <Button 
          variant="outline-secondary" 
          size="sm" 
          onClick={() => handleAction('CANCEL')}
          disabled={isProcessing}
        >
          ↩ Hủy Bỏ (Quay Lại)
        </Button>

        <div className="d-flex flex-wrap gap-2">
          {/* Lựa chọn 2: Giữ nguyên Cloud SL */}
          <Button 
            variant="outline-warning" 
            size="sm" 
            className="fw-semibold text-uppercase"
            onClick={() => handleAction('LEAVE_CLOUD_SL')}
            disabled={isProcessing}
          >
            {isProcessing && selectedAction === 'LEAVE_CLOUD_SL' 
              ? 'Đang Đóng App...' 
              : '🛡️ Giữ Nguyên SL & Thoát'}
          </Button>

          {/* Lựa chọn 1: Đóng sạch Market */}
          <Button 
            variant="danger" 
            size="sm" 
            className="fw-bold text-uppercase shadow-sm"
            onClick={() => handleAction('FLAT_ALL')}
            disabled={isProcessing}
          >
            {isProcessing && selectedAction === 'FLAT_ALL' 
              ? 'Đang Đóng Market...' 
              : '⚡ Đóng Sạch Market & Thoát'}
          </Button>
        </div>
      </Modal.Footer>
    </Modal>
  );
};
