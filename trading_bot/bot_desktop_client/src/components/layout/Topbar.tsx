// trading_bot/bot_desktop_client/src/components/layout/Topbar.tsx
import React from 'react';
import { Navbar, Container, Badge, Button } from 'react-bootstrap';
import { isTauriEnvironment } from '../../services/botService';

interface TopbarProps {
  wsConnected?: boolean;
  onTriggerExit?: () => void;
}

export const Topbar: React.FC<TopbarProps> = ({ 
  wsConnected = true,
  onTriggerExit,
}) => {
  const isTauri = isTauriEnvironment();

  return (
    <Navbar bg="black" variant="dark" className="border-bottom border-secondary py-2 shadow-sm">
      <Container fluid className="d-flex justify-content-between align-items-center">
        {/* Brand Logo & Title */}
        <Navbar.Brand href="#home" className="fw-bold fs-5 text-info d-flex align-items-center gap-2">
          <span>🚀</span>
          <span className="tracking-wide">FERRO-BOT QUANT DESK</span>
          <small className="text-secondary fs-6 fw-normal ms-2">v1.0</small>
        </Navbar.Brand>

        {/* Status Indicators & Action Group */}
        <div className="d-flex align-items-center gap-3">
          {/* Demo Mode Badge */}
          <Badge bg="warning" text="dark" className="px-3 py-2 fw-semibold border border-warning">
            🧪 DEMO MODE (Binance Testnet)
          </Badge>

          {/* Engine IPC Connection Indicator */}
          <div className="d-flex align-items-center gap-2 px-3 py-1 bg-dark border border-secondary rounded-pill">
            <span 
              className={`d-inline-block rounded-circle ${wsConnected ? 'bg-success' : 'bg-danger'}`}
              style={{ width: '8px', height: '8px' }}
            />
            <span className="small text-light fw-medium">
              {isTauri 
                ? (wsConnected ? 'RUST ENGINE CONNECTED' : 'ENGINE DISCONNECTED') 
                : 'BROWSER MOCK MODE'}
            </span>
          </div>

          {/* Nút Thoát Ứng Dụng (Kích hoạt Unified Exit) */}
          {onTriggerExit && (
            <Button
              variant="outline-danger"
              size="sm"
              className="fw-bold px-3 py-1"
              title="Thoát hoàn toàn ứng dụng (Unified Exit)"
              onClick={onTriggerExit}
            >
              ✕ Thoát App
            </Button>
          )}
        </div>
      </Container>
    </Navbar>
  );
};
