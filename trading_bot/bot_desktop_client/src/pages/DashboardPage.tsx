// trading_bot/bot_desktop_client/src/pages/DashboardPage.tsx
import React from 'react';
import { Container } from 'react-bootstrap';
import { AccountState, SystemHealth } from '../types/bot';
import { KpiSummaryCards } from '../components/dashboard/KpiSummaryCards';
import { SystemHealthBar } from '../components/dashboard/SystemHealthBar';
import { GlobalKillSwitch } from '../components/dashboard/GlobalKillSwitch';

interface DashboardPageProps {
  account: AccountState;
  health: SystemHealth;
  totalAssetsCount: number;
  onEmergencyStopAll: () => Promise<void>;
}

export const DashboardPage: React.FC<DashboardPageProps> = ({
  account,
  health,
  totalAssetsCount,
  onEmergencyStopAll,
}) => {
  return (
    <Container fluid className="p-4">
      {/* 4 Thẻ KPI Summary Overview */}
      <KpiSummaryCards 
        account={account} 
        totalAssetsCount={totalAssetsCount} 
      />

      {/* Thanh Giám sát Sức khỏe Rust Engine & Connection */}
      <SystemHealthBar 
        health={health} 
      />

      {/* Global Emergency Kill Switch */}
      <GlobalKillSwitch 
        onEmergencyStopAll={onEmergencyStopAll}
        activeAssetsCount={account.activeAssetsCount}
      />
    </Container>
  );
};
