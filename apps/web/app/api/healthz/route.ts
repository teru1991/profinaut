import { NextResponse } from "next/server";

const API_URL = process.env.DASHBOARD_API_URL ?? "http://localhost:8000";

export async function GET() {
  try {
    const response = await fetch(`${API_URL}/healthz`, { cache: "no-store" });
    const payload = await response.text();
    return new NextResponse(payload, {
      status: response.status,
      headers: {
        "content-type": response.headers.get("content-type") ?? "application/json"
      }
    });
  } catch {
    return NextResponse.json({ status: "down" }, { status: 503 });
  }
}
