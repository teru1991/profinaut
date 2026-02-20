"use client";

import { Suspense } from "react";
import DashboardWorkspace from "../../components/dashboard/DashboardWorkspace";

export default function DashboardPage() {
  return (
    <Suspense fallback={<div className="card">Loading dashboard workspace…</div>}>
      <DashboardWorkspace />
    </Suspense>
  );
}
