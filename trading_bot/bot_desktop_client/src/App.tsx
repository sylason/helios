import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Container, Table, Button, Badge, Card, Navbar } from "react-bootstrap";

function App() {
  // Trạng thái cho từng Asset (Trong demo này chỉ có ETHUSDT)
  const [signalReady, setSignalReady] = useState(false);
  const [isActivating, setIsActivating] = useState(false);

  const handleActivate = async (asset: string) => {
    if (!isActivating) {
      // Logic BẬT: Sync 30 ngày -> Optimize ngầm -> Signal READY -> Trade
      setIsActivating(true);
      setSignalReady(false);
      
      try {
        await invoke("cmd_activate_asset", { asset });
        // Giả lập chu kỳ Sync & Optimize ban đầu mất khoảng 3 giây
        // Trong thực tế, Backend sẽ gửi event khi Ready, nhưng ở demo ta dùng timeout
        setTimeout(() => setSignalReady(true), 3000);
      } catch (error) {
        console.error("Lỗi kích hoạt:", error);
        setIsActivating(false);
      }
    } else {
      // Logic TẮT (Immediate): Đóng sạch mọi thứ ngay lập tức
      setIsActivating(false);
      setSignalReady(false);
      await invoke("cmd_terminate_immediate", { asset });
    }
  };

  const handleGraceful = async (asset: string) => {
    // Logic DỪNG AN TOÀN: Chặn mở lệnh mới, đợi Trailing SL đóng lệnh cũ
    await invoke("cmd_terminate_graceful", { asset });
  };

  return (
    <div className="bg-dark min-vh-100 text-light">
      <Navbar bg="black" variant="dark" className="border-bottom border-secondary mb-4">
        <Container fluid>
          <Navbar.Brand href="#home" className="fw-bold text-info">
            🚀 Trading Bot Client v1
          </Navbar.Brand>
          <Navbar.Text>
            Mode: <Badge bg="warning" text="dark">DEMO (Binance Testnet)</Badge>
          </Navbar.Text>
        </Container>
      </Navbar>

      <Container fluid>
        <Card bg="dark" border="secondary" className="shadow-sm">
          <Card.Header className="bg-black text-info border-secondary py-3">
            <h5 className="mb-0">Asset Management (Active Watchlist)</h5>
          </Card.Header>
          <Card.Body className="p-0">
            <Table responsive variant="dark" hover className="mb-0 align-middle text-center border-0">
              <thead className="text-secondary text-uppercase small">
                <tr>
                  <th>Category</th>
                  <th>Exchange</th>
                  <th>Asset Symbol</th>
                  <th style={{ width: '250px' }}>Main Action</th>
                  <th style={{ width: '250px' }}>Safe Exit</th>
                  <th>Signal Status</th>
                </tr>
              </thead>
              <tbody>
                <tr>
                  <td><Badge bg="outline-secondary" className="border border-secondary text-secondary">FUTURES</Badge></td>
                  <td className="text-secondary">BINANCE</td>
                  <td className="fw-bold">ETHUSDT</td>
                  <td>
                    <Button 
                      variant={isActivating ? "outline-danger" : "outline-success"} 
                      className="w-100 py-2 fw-bold"
                      onClick={() => handleActivate("ETHUSDT")}
                    >
                      {isActivating ? "DEACTIVATE (IMMEDIATE)" : "ACTIVATE BOT"}
                    </Button>
                  </td>
                  <td>
                    <Button 
                      variant="outline-warning" 
                      className="w-100 py-2 fw-bold"
                      onClick={() => handleGraceful("ETHUSDT")}
                      disabled={!isActivating}
                    >
                      TERMINATE GRACEFULLY
                    </Button>
                  </td>
                  <td>
                    {isActivating ? (
                      <Badge bg={signalReady ? "success" : "warning"} className="px-3 py-2">
                        {signalReady ? "READY TO TRADE" : "SYNCING / OPTIMIZING..."}
                      </Badge>
                    ) : (
                      <Badge bg="secondary" className="px-3 py-2 text-dark" style={{ opacity: 0.5 }}>
                        INACTIVE
                      </Badge>
                    )}
                  </td>
                </tr>
              </tbody>
            </Table>
          </Card.Body>
        </Card>

        <footer className="mt-4 text-secondary small">
          <p className="mb-0">* Hệ thống tự động duy trì cửa sổ 30 ngày dữ liệu nến 1m cho các Asset được Activate.</p>
          <p>* Mọi quyết định Optimize và Hot-swap DNA diễn ra hoàn toàn trong Engine ngầm.</p>
        </footer>
      </Container>
    </div>
  );
}

export default App;
