// trading_bot/bot_desktop_client/src/components/control/AssetLogPanel.tsx
import React from 'react';
import { Card, Badge } from 'react-bootstrap';

interface AssetLogPanelProps {
  symbol: string;
  logs: string[];
}

export const AssetLogPanel: React.FC<AssetLogPanelProps> = ({ symbol, logs }) => {
  const recentLogs = logs.slice(-5); // Lấy tối đa 5 log gần nhất

  return (
    <Card bg="dark" border="secondary" className="shadow-sm mt-3">
      <Card.Header className="bg-black text-info border-secondary py-2.5 d-flex justify-content-between align-items-center">
        <div className="d-flex align-items-center gap-2">
          <span className="fs-6">📋</span>
          <h6 className="mb-0 fw-bold">Recent Event Stream Logs: <span className="text-warning">{symbol}</span></h6>
        </div>
        <Badge bg="secondary" className="small">Showing last {recentLogs.length} events</Badge>
      </Card.Header>

      <Card.Body className="bg-black p-3 font-monospace small" style={{ minHeight: '140px', maxHeight: '200px', overflowY: 'auto' }}>
        {recentLogs.length === 0 ? (
          <div className="text-secondary italic text-center py-4 opacity-50">
            No recent events recorded for {symbol}. Activate bot to start streaming events.
          </div>
        ) : (
          <div className="d-flex flex-column gap-1">
            {recentLogs.map((log, index) => (
              <div key={index} className="text-light d-flex align-items-start gap-2">
                <span className="text-secondary opacity-75 select-none">[{index + 1}]</span>
                <span className="text-info opacity-90">{log}</span>
              </div>
            ))}
          </div>
        )}
      </Card.Body>
    </Card>
  );
};
