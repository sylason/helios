// trading_bot/bot_desktop_client/src/components/dashboard/GlobalKillSwitch.tsx
import React, { useState } from 'react';
import { Card, Button, Modal } from 'react-bootstrap';

interface GlobalKillSwitchProps {
  onEmergencyStopAll: () => Promise<void>;
  activeAssetsCount: number;
}

export const GlobalKillSwitch: React.FC<GlobalKillSwitchProps> = ({
  onEmergencyStopAll,
  activeAssetsCount,
}) => {
  const [showConfirmModal, setShowConfirmModal] = useState(false);
  const [isStopping, setIsStopping] = useState(false);

  const handleConfirmStop = async () => {
    setIsStopping(true);
    try {
      await onEmergencyStopAll();
    } finally {
      setIsStopping(false);
      setShowConfirmModal(false);
    }
  };

  return (
    <>
      <Card bg="dark" border="danger" className="shadow-lg border-2">
        <Card.Body className="p-4 bg-black rounded text-center">
          <div className="d-flex align-items-center justify-content-center gap-2 mb-2 text-danger">
            <span className="fs-3">🚨</span>
            <h5 className="mb-0 fw-bold text-uppercase tracking-wider">
              Emergency Kill-Switch (Global Command)
            </h5>
          </div>

          <p className="text-secondary small mb-4 mx-auto" style={{ maxWidth: '600px' }}>
            Hủy tức thì tất cả các lệnh Algo chờ xử lý và đóng toàn bộ vị thế đang chạy của tất cả {activeAssetsCount} Cặp tiền trên Binance Futures Testnet.
          </p>

          <Button 
            variant="danger" 
            size="lg"
            className="px-5 py-3 fw-bold text-uppercase shadow-lg tracking-wider"
            onClick={() => setShowConfirmModal(true)}
            disabled={activeAssetsCount === 0 || isStopping}
          >
            🚨 EMERGENCY STOP ALL BOTS ({activeAssetsCount})
          </Button>
        </Card.Body>
      </Card>

      {/* Modal xác nhận Khẩn cấp */}
      <Modal 
        show={showConfirmModal} 
        onHide={() => setShowConfirmModal(false)}
        centered
        contentClassName="bg-dark text-light border-danger border-2 shadow-lg"
      >
        <Modal.Header closeButton closeVariant="white" className="border-secondary bg-black">
          <Modal.Title className="text-danger fw-bold d-flex align-items-center gap-2">
            <span>🚨</span> XÁC NHẬN DỪNG KHẨN CẤP TOÀN CỤC
          </Modal.Title>
        </Modal.Header>

        <Modal.Body className="py-4">
          <p className="fw-semibold text-warning">
            Bạn có chắc chắn muốn ngắt khẩn cấp tất cả {activeAssetsCount} Bot đang hoạt động không?
          </p>
          <ul className="text-secondary small">
            <li>Tất cả lệnh Trailing SL / Hard SL đang treo sẽ bị HỦY lập tức.</li>
            <li>Vị thế hiện tại sẽ dừng lại và ngắt kết nối Engine.</li>
          </ul>
        </Modal.Body>

        <Modal.Footer className="border-secondary bg-black">
          <Button variant="outline-secondary" onClick={() => setShowConfirmModal(false)}>
            Hủy Bỏ
          </Button>
          <Button 
            variant="danger" 
            className="fw-bold px-4"
            onClick={handleConfirmStop}
            disabled={isStopping}
          >
            {isStopping ? 'ĐANG DỪNG...' : 'XÁC NHẬN DỪNG TOÀN BỘ'}
          </Button>
        </Modal.Footer>
      </Modal>
    </>
  );
};
