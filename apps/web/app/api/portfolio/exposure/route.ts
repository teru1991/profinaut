import { NextResponse } from "next/server";

const API_URL = process.env.DASHBOARD_API_URL ?? "http://localhost:8000";
const ADMIN_TOKEN = process.env.ADMIN_TOKEN ?? "";

export async function GET() {
  try {
    const response = await fetch(`${API_URL}/portfolio/exposure`, {
      headers: {
        "X-Admin-Token": ADMIN_TOKEN
      },
      cache: "no-store"
    });

    const payload = await response.text();
    return new NextResponse(payload, {
      status: response.status,
      headers: {
        "content-type": response.headers.get("content-type") ?? "application/json"
      }
    });
  } catch {
    return NextResponse.json(
      {
        generated_at: new Date().toISOString(),
        total_net_exposure: 0,
        total_gross_exposure: 0,
        key_metrics: { latest_equity: 0, tracked_positions: 0, tracked_symbols: 0 },
        by_symbol: []
      },
      { status: 200 }
    );
  }
}
