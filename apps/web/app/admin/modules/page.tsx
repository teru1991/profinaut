export default function AdminModulesPage() {
  return (
    <div>
      <div className="page-header">
        <div className="page-header-left">
          <h1 className="page-title">Admin / Modules</h1>
          <p className="page-subtitle">Module registry, lifecycle controls, and configuration</p>
        </div>
      </div>
      <div className="card">
        <div className="placeholder-page">
          <div className="placeholder-icon">&#x2699;&#xFE0F;</div>
          <h2 className="placeholder-title">Module Registry</h2>
          <p className="placeholder-description">
            View registered modules, their run history, success rates, and
            toggle module execution from this admin panel.
          </p>
          <div className="notice notice-info" style={{ maxWidth: 420 }}>
            <span className="notice-icon">i</span>
            <div className="notice-content">
              Module controls are wired in the backend and will be surfaced here
              incrementally. Includes run stats, failure rates, and stuck-run detection.
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
