export default function DatasetsPage() {
  return (
    <div>
      <div className="page-header">
        <div className="page-header-left">
          <h1 className="page-title">Datasets</h1>
          <p className="page-subtitle">Dataset catalog, Parquet metadata, and quality summaries</p>
        </div>
      </div>
      <div className="card">
        <div className="placeholder-page">
          <div className="placeholder-icon">&#x1F4BE;</div>
          <h2 className="placeholder-title">Datasets Coming Soon</h2>
          <p className="placeholder-description">
            Browse, inspect, and validate dataset files. Parquet schema previews
            and data quality checks will be available in upcoming steps.
          </p>
          <div className="notice notice-info" style={{ maxWidth: 420 }}>
            <span className="notice-icon">i</span>
            <div className="notice-content">
              The dataset catalog will integrate with the backend dataset registry
              to provide schema inspection, row counts, and freshness signals.
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
