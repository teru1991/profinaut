import { NextResponse } from "next/server";

const DASHBOARD_API_BASE_URL =
  process.env.DASHBOARD_API_BASE_URL ?? process.env.DASHBOARD_API_URL ?? "http://localhost:8000";
const DASHBOARD_ADMIN_TOKEN = process.env.DASHBOARD_ADMIN_TOKEN ?? process.env.ADMIN_TOKEN ?? "test-admin-token";
const STATUS_TIMEOUT_MS = 3000;

function normalizeBaseUrl(url: string): string {
  return url.replace(/\/+$/, "");
}

export async function GET() {
  const upstreamBaseUrl = normalizeBaseUrl(DASHBOARD_API_BASE_URL);
  const upstreamUrl = `${upstreamBaseUrl}/api/status/summary`;

  const controller = new AbortController();
  const timeoutId = setTimeout(() => controller.abort(), STATUS_TIMEOUT_MS);

  try {
    const response = await fetch(upstreamUrl, {
      headers: {
        "X-Admin-Token": DASHBOARD_ADMIN_TOKEN
      },
      cache: "no-store",
      signal: controller.signal
    });

    const text = await response.text();
    return new NextResponse(text, {
      status: response.status,
      headers: {
        "content-type": response.headers.get("content-type") ?? "application/json"
      }
    });
  } catch (error) {
    const message =
      error instanceof Error && error.name === "AbortError"
        ? "request timed out while contacting status backend"
        : "failed to reach status backend";
    return NextResponse.json(
      {
        error: "bad_gateway",
        message
      },
      { status: 502 }
    );
  } finally {
    clearTimeout(timeoutId);
  }
}
