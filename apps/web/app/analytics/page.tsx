export default function AnalyticsPage() {
  return (
    <div>
      <div className="page-header">
        <div className="page-header-left">
          <h1 className="page-title">Analytics</h1>
          <p className="page-subtitle">Performance, execution quality, and NetPnL metrics</p>
        </div>
      </div>
      <div className="card">
        <div className="placeholder-page">
          <div className="placeholder-icon">&#x1F4CA;</div>
          <h2 className="placeholder-title">Analytics Coming Soon</h2>
          <p className="placeholder-description">
            NetPnL, execution quality, drawdown analysis, and module performance
            analytics will be surfaced here once the metrics pipeline is fully wired.
          </p>
          <div className="notice notice-info" style={{ maxWidth: 420 }}>
            <span className="notice-icon">i</span>
            <div className="notice-content">
              Metrics are already being collected by the backend. This view will
              visualize equity curves, slippage, fill ratios, and resource telemetry.
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
