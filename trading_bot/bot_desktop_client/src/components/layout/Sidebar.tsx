// trading_bot/bot_desktop_client/src/components/layout/Sidebar.tsx
import React from 'react';
import { Nav, Badge } from 'react-bootstrap';
import { PageId } from '../../types/bot';

interface SidebarProps {
  activePage: PageId;
  onSelectPage: (page: PageId) => void;
}

export const Sidebar: React.FC<SidebarProps> = ({ activePage, onSelectPage }) => {
  return (
    <div 
      className="bg-black border-end border-secondary d-flex flex-column py-3 px-2 shadow-sm"
      style={{ width: '240px', minHeight: 'calc(100vh - 56px)' }}
    >
      <div className="text-uppercase text-secondary px-3 mb-2 small fw-bold tracking-wider">
        Main Navigation
      </div>

      <Nav className="flex-column gap-1">
        {/* Trang 2: Bot Control Center (Priority #1) */}
        <Nav.Link
          active={activePage === 'control'}
          onClick={() => onSelectPage('control')}
          className={`d-flex align-items-center justify-content-between px-3 py-2.5 rounded text-start fw-medium ${
            activePage === 'control' 
              ? 'bg-info text-dark fw-bold shadow-sm' 
              : 'text-light hover-bg-dark'
          }`}
          style={{ cursor: 'pointer' }}
        >
          <div className="d-flex align-items-center gap-2">
            <span>⚙️</span>
            <span>Bot Control</span>
          </div>
          <Badge bg={activePage === 'control' ? 'dark' : 'info'} text={activePage === 'control' ? 'info' : 'dark'}>
            P1
          </Badge>
        </Nav.Link>

        {/* Trang 1: Executive Dashboard (Priority #2) */}
        <Nav.Link
          active={activePage === 'dashboard'}
          onClick={() => onSelectPage('dashboard')}
          className={`d-flex align-items-center justify-content-between px-3 py-2.5 rounded text-start fw-medium ${
            activePage === 'dashboard' 
              ? 'bg-info text-dark fw-bold shadow-sm' 
              : 'text-light hover-bg-dark'
          }`}
          style={{ cursor: 'pointer' }}
        >
          <div className="d-flex align-items-center gap-2">
            <span>📊</span>
            <span>Dashboard</span>
          </div>
          <Badge bg={activePage === 'dashboard' ? 'dark' : 'secondary'}>
            P2
          </Badge>
        </Nav.Link>

        <hr className="border-secondary my-3 opacity-50" />

        <div className="text-uppercase text-secondary px-3 mb-2 small fw-bold tracking-wider">
          Deferred (Post-Demo)
        </div>

        {/* Disabled Items */}
        <Nav.Link disabled className="d-flex align-items-center justify-content-between px-3 py-2 text-secondary opacity-50">
          <div className="d-flex align-items-center gap-2">
            <span>🔒</span>
            <span>Settings & API</span>
          </div>
          <small className="italic opacity-75">Post-Demo</small>
        </Nav.Link>

        <Nav.Link disabled className="d-flex align-items-center justify-content-between px-3 py-2 text-secondary opacity-50">
          <div className="d-flex align-items-center gap-2">
            <span>🔒</span>
            <span>Trade Journal</span>
          </div>
          <small className="italic opacity-75">Post-Demo</small>
        </Nav.Link>

        <Nav.Link disabled className="d-flex align-items-center justify-content-between px-3 py-2 text-secondary opacity-50">
          <div className="d-flex align-items-center gap-2">
            <span>🔒</span>
            <span>Live Charts</span>
          </div>
          <small className="italic opacity-75">Post-Demo</small>
        </Nav.Link>
      </Nav>

      <div className="mt-auto px-3 pt-3 border-top border-secondary text-secondary small">
        <p className="mb-1 text-light fw-semibold">Local-First Architecture</p>
        <p className="mb-0 text-muted" style={{ fontSize: '0.75rem' }}>
          Trading Bot Client v1.0
        </p>
      </div>
    </div>
  );
};
