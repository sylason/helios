// trading_bot/bot_desktop_client/src/components/dashboard/KpiSummaryCards.tsx
import React from 'react';
import { Row, Col, Card } from 'react-bootstrap';
import { AccountState } from '../../types/bot';
import { formatCurrency, formatPercent } from '../../utils/formatters';

interface KpiSummaryCardsProps {
  account: AccountState;
  totalAssetsCount: number;
}

export const KpiSummaryCards: React.FC<KpiSummaryCardsProps> = ({ account, totalAssetsCount }) => {
  const isPnlPositive = account.totalUnrealizedPnl >= 0;

  return (
    <Row className="g-3 mb-4">
      {/* Wallet Balance Card */}
      <Col md={3}>
        <Card bg="dark" border="secondary" className="shadow-sm h-100">
          <Card.Body className="d-flex flex-column justify-content-between">
            <div className="text-secondary small text-uppercase fw-bold tracking-wider mb-1">
              💳 Total Wallet Balance
            </div>
            <div className="fs-3 fw-bold text-light">
              {formatCurrency(account.walletBalance)}
            </div>
            <small className="text-secondary mt-1">Binance Futures Testnet</small>
          </Card.Body>
        </Card>
      </Col>

      {/* Available Balance Card */}
      <Col md={3}>
        <Card bg="dark" border="secondary" className="shadow-sm h-100">
          <Card.Body className="d-flex flex-column justify-content-between">
            <div className="text-secondary small text-uppercase fw-bold tracking-wider mb-1">
              💵 Available Margin
            </div>
            <div className="fs-3 fw-bold text-info">
              {formatCurrency(account.availableBalance)}
            </div>
            <small className="text-secondary mt-1">Available for new orders</small>
          </Card.Body>
        </Card>
      </Col>

      {/* Total uPnL Card */}
      <Col md={3}>
        <Card bg="dark" border="secondary" className="shadow-sm h-100">
          <Card.Body className="d-flex flex-column justify-content-between">
            <div className="text-secondary small text-uppercase fw-bold tracking-wider mb-1">
              📈 Total Unrealized PnL
            </div>
            <div className={`fs-3 fw-bold ${isPnlPositive ? 'text-success' : 'text-danger'}`}>
              {formatCurrency(account.totalUnrealizedPnl)}
            </div>
            <small className={isPnlPositive ? 'text-success fw-bold' : 'text-danger fw-bold'}>
              {formatPercent(account.walletBalance > 0 ? (account.totalUnrealizedPnl / account.walletBalance) * 100 : 0)} Total Return
            </small>
          </Card.Body>
        </Card>
      </Col>

      {/* Active Assets Count Card */}
      <Col md={3}>
        <Card bg="dark" border="secondary" className="shadow-sm h-100">
          <Card.Body className="d-flex flex-column justify-content-between">
            <div className="text-secondary small text-uppercase fw-bold tracking-wider mb-1">
              ⚡ Active Trading Bots
            </div>
            <div className="fs-3 fw-bold text-warning">
              {account.activeAssetsCount} <span className="fs-6 text-secondary fw-normal">/ {totalAssetsCount} Assets</span>
            </div>
            <small className="text-secondary mt-1">Live background instances</small>
          </Card.Body>
        </Card>
      </Col>
    </Row>
  );
};
