// trading_bot/bot_desktop_client/src/components/control/RebalanceModal.tsx
import React, { useState, useEffect } from 'react';
import { Modal, Button, Form, Alert, Card } from 'react-bootstrap';
import { AssetState, RebalanceOption } from '../../types/bot';
import { formatCurrency } from '../../utils/formatters';

interface RebalanceModalProps {
  show: boolean;
  onHide: () => void;
  sourceAsset: AssetState | null;
  allAssets: AssetState[];
  onConfirmRebalance: (
    fromSymbol: string,
    toSymbol: string,
    amount: number,
    option: RebalanceOption
  ) => Promise<void>;
}

export const RebalanceModal: React.FC<RebalanceModalProps> = ({
  show,
  onHide,
  sourceAsset,
  allAssets,
  onConfirmRebalance,
}) => {
  const [targetSymbol, setTargetSymbol] = useState<string>('');
  const [amount, setAmount] = useState<number>(0);
  const [option, setOption] = useState<RebalanceOption>('APPLY_NEXT_TRADE');
  const [isSubmitting, setIsSubmitting] = useState<boolean>(false);

  // Danh sách các cặp tiền đích có thể nhận vốn (loại trừ cặp nguồn)
  const availableTargetAssets = allAssets.filter(
    (a) => a.symbol !== sourceAsset?.symbol
  );

  // Khởi tạo giá trị mặc định khi mở modal
  useEffect(() => {
    if (sourceAsset) {
      if (availableTargetAssets.length > 0) {
        setTargetSymbol(availableTargetAssets[0].symbol);
      }
      // Mặc định đề xuất số tiền lãi thực tế (nếu có lãi) hoặc $20.0
      const defaultAmount = sourceAsset.realizedPnl > 0 
        ? sourceAsset.realizedPnl 
        : Math.min(20.0, sourceAsset.allocatedCapital);
      setAmount(Number(defaultAmount.toFixed(2)));
      setOption('APPLY_NEXT_TRADE');
      setIsSubmitting(false);
    }
  }, [sourceAsset, show]);

  if (!sourceAsset) return null;

  const isInPosition = sourceAsset.position.side !== 'FLAT';
  const maxTransferable = Math.max(0, sourceAsset.allocatedCapital - 10.0); // Giữ lại tối thiểu $10 làm vốn sàn

  const handleFormSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (amount <= 0 || amount > maxTransferable || !targetSymbol) return;

    setIsSubmitting(true);
    try {
      await onConfirmRebalance(sourceAsset.symbol, targetSymbol, amount, option);
      onHide();
    } catch (err) {
      console.error('Lỗi khi điều chuyển vốn:', err);
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <Modal show={show} onHide={onHide} centered backdrop="static" contentClassName="bg-dark text-light border border-secondary shadow-lg">
      <Modal.Header closeButton closeVariant="white" className="border-secondary bg-black">
        <Modal.Title className="fs-5 fw-bold text-info d-flex align-items-center gap-2">
          <span>⚙️ Điều Chuyển Vốn An Toàn (Capital Rebalance)</span>
        </Modal.Title>
      </Modal.Header>

      <Form onSubmit={handleFormSubmit}>
        <Modal.Body className="py-3">
          {/* 1. Thông Tin Cặp Nguồn & Số Dư Vốn */}
          <Card bg="black" border="secondary" className="mb-3 p-3">
            <div className="d-flex justify-content-between align-items-center mb-2">
              <span className="text-secondary small">Cặp Tiền Nguồn:</span>
              <span className="fw-bold fs-6 text-light">{sourceAsset.symbol}</span>
            </div>
            <div className="d-flex justify-content-between align-items-center mb-1">
              <span className="text-secondary small">Tổng Vốn Phân Bổ:</span>
              <span className="fw-bold text-info">{formatCurrency(sourceAsset.allocatedCapital)}</span>
            </div>
            <div className="d-flex justify-content-between align-items-center">
              <span className="text-secondary small">Lãi Thực Tế (Realized PnL):</span>
              <span className={sourceAsset.realizedPnl >= 0 ? 'text-success fw-bold' : 'text-danger fw-bold'}>
                {formatCurrency(sourceAsset.realizedPnl)}
              </span>
            </div>
          </Card>

          {/* 2. Chọn Cặp Tiền Đích & Nhập Số Tiền */}
          <div className="mb-3">
            <Form.Label className="small text-secondary fw-semibold">Chuyển Vốn Sang Cặp Tiền:</Form.Label>
            <Form.Select
              value={targetSymbol}
              onChange={(e) => setTargetSymbol(e.target.value)}
              className="bg-dark text-light border-secondary shadow-none"
            >
              {availableTargetAssets.map((asset) => (
                <option key={asset.symbol} value={asset.symbol}>
                  {asset.symbol} (Vốn hiện tại: {formatCurrency(asset.allocatedCapital)})
                </option>
              ))}
            </Form.Select>
          </div>

          <div className="mb-3">
            <div className="d-flex justify-content-between align-items-center mb-1">
              <Form.Label className="small text-secondary fw-semibold mb-0">Số Tiền Muốn Chuyển (USDT):</Form.Label>
              <small className="text-info cursor-pointer" onClick={() => setAmount(Number(maxTransferable.toFixed(2)))} style={{ cursor: 'pointer' }}>
                Tối đa: {formatCurrency(maxTransferable)}
              </small>
            </div>
            <Form.Control
              type="number"
              step="0.01"
              min="1"
              max={maxTransferable}
              value={amount}
              onChange={(e) => setAmount(parseFloat(e.target.value) || 0)}
              className="bg-dark text-light border-secondary shadow-none fw-bold"
              required
            />
          </div>

          {/* 3. In-Position Guard: Phân Nhánh An Toàn Nếu Đang Mở Vị Thế */}
          {isInPosition ? (
            <Alert variant="warning" className="border-warning bg-opacity-10 bg-warning text-light py-2 px-3 mb-0">
              <div className="d-flex align-items-center gap-2 mb-2">
                <span className="fs-5">⚠️</span>
                <span className="fw-bold text-warning small">
                  {sourceAsset.symbol} ĐANG CÓ VỊ THẾ MỞ ({sourceAsset.position.side} {sourceAsset.position.qty})
                </span>
              </div>
              <p className="small text-secondary mb-2">
                Rút vốn ngay trong lúc gồng lệnh có thể làm tăng đòn bẩy thực tế. Vui lòng chọn cách xử lý an toàn:
              </p>

              <Form.Check
                type="radio"
                id="opt-next-trade"
                name="rebalance-option"
                label={
                  <div className="small">
                    <span className="fw-bold text-success">🟢 Áp Dụng Từ Lệnh Kế Tiếp (Khuyến Nghị):</span>
                    <div className="text-secondary">Chờ lệnh hiện tại chốt xong SL/TP thì hệ thống mới tự động chuyển vốn. Vị thế đang chạy được bảo toàn 100%.</div>
                  </div>
                }
                checked={option === 'APPLY_NEXT_TRADE'}
                onChange={() => setOption('APPLY_NEXT_TRADE')}
                className="mb-2"
              />

              <Form.Check
                type="radio"
                id="opt-close-now"
                name="rebalance-option"
                label={
                  <div className="small">
                    <span className="fw-bold text-danger">⚡ Đóng Vị Thế Ngay Lập Tức:</span>
                    <div className="text-secondary">Gửi lệnh Market chốt vị thế ngay tức khắc và chuyển vốn sang {targetSymbol} ngay bây giờ.</div>
                  </div>
                }
                checked={option === 'CLOSE_IMMEDIATELY'}
                onChange={() => setOption('CLOSE_IMMEDIATELY')}
              />
            </Alert>
          ) : (
            <div className="p-2 bg-black border border-secondary rounded small text-success d-flex align-items-center gap-2">
              <span>✅</span>
              <span>{sourceAsset.symbol} đang ở trạng thái FLAT (không có lệnh). Vốn sẽ được cập nhật tức thì.</span>
            </div>
          )}
        </Modal.Body>

        <Modal.Footer className="border-secondary bg-black">
          <Button variant="outline-secondary" size="sm" onClick={onHide} disabled={isSubmitting}>
            Hủy Bỏ
          </Button>
          <Button 
            type="submit" 
            variant="info" 
            size="sm" 
            className="fw-bold text-dark px-3"
            disabled={isSubmitting || amount <= 0 || amount > maxTransferable || !targetSymbol}
          >
            {isSubmitting ? 'Đang Xử Lý...' : 'Xác Nhận Điều Chuyển'}
          </Button>
        </Modal.Footer>
      </Form>
    </Modal>
  );
};
