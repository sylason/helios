// trading_bot/bot_desktop_client/src/components/dashboard/SystemHealthBar.tsx
import React from 'react';
import { Card, ProgressBar, Row, Col, Badge } from 'react-bootstrap';
import { SystemHealth } from '../../types/bot';

interface SystemHealthBarProps {
  health: SystemHealth;
}

export const SystemHealthBar: React.FC<SystemHealthBarProps> = ({ health }) => {
  const weightPct = Math.min(100, Math.round((health.apiWeightUsed / health.apiWeightMax) * 100));
  
  let progressBarVariant = 'success';
  if (weightPct > 80) progressBarVariant = 'danger';
  else if (weightPct > 50) progressBarVariant = 'warning';

  return (
    <Card bg="dark" border="secondary" className="shadow-sm mb-4">
      <Card.Header className="bg-black text-info border-secondary py-2.5 d-flex justify-content-between align-items-center">
        <div className="d-flex align-items-center gap-2">
          <span>🩺</span>
          <h6 className="mb-0 fw-bold">Rust Engine System Health & Connection Monitor</h6>
        </div>
        <Badge bg="success" className="px-3 py-1">OPERATIONAL</Badge>
      </Card.Header>

      <Card.Body className="p-3">
        <Row className="align-items-center g-3">
          {/* Binance API Rate Limit Weight */}
          <Col md={6}>
            <div className="d-flex justify-content-between align-items-center mb-1">
              <span className="small text-secondary fw-semibold">Binance REST API Weight Limit</span>
              <span className="small text-light font-monospace fw-bold">
                {health.apiWeightUsed} / {health.apiWeightMax} ({weightPct}%)
              </span>
            </div>
            <ProgressBar 
              now={weightPct} 
              variant={progressBarVariant} 
              style={{ height: '8px' }} 
              className="bg-black border border-secondary"
            />
          </Col>

          {/* WebSocket Connection Latency */}
          <Col md={3}>
            <div className="d-flex align-items-center gap-2 p-2 bg-black border border-secondary rounded">
              <span 
                className={`d-inline-block rounded-circle ${health.wsConnected ? 'bg-success' : 'bg-danger'}`}
                style={{ width: '10px', height: '10px' }}
              />
              <div>
                <div className="small text-secondary leading-none">WebSocket Kline Stream</div>
                <div className="small text-light fw-bold">{health.latencyMs} ms Latency</div>
              </div>
            </div>
          </Col>

          {/* Local SQLite Database Engine */}
          <Col md={3}>
            <div className="d-flex align-items-center gap-2 p-2 bg-black border border-secondary rounded">
              <span className="fs-6">💾</span>
              <div>
                <div className="small text-secondary leading-none">Local Data Engine</div>
                <div className="small text-info fw-bold">SQLite Store Active</div>
              </div>
            </div>
          </Col>
        </Row>
      </Card.Body>
    </Card>
  );
};
