import { BotsTable } from "../../components/BotsTable";

export default function BotsPage() {
  return (
    <div>
      <div className="page-header">
        <div className="page-header-left">
          <h1 className="page-title">Bots</h1>
          <p className="page-subtitle">Monitor registered bots, their health, and heartbeat status</p>
        </div>
      </div>
      <BotsTable />
    </div>
  );
}
