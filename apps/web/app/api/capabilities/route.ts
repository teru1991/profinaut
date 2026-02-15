import { NextResponse } from "next/server";

const DASHBOARD_API_BASE_URL =
  process.env.DASHBOARD_API_BASE_URL ?? process.env.DASHBOARD_API_URL ?? "http://localhost:8000";
const DASHBOARD_ADMIN_TOKEN = process.env.DASHBOARD_ADMIN_TOKEN ?? process.env.ADMIN_TOKEN ?? "test-admin-token";
const CAPABILITIES_TIMEOUT_MS = 4000;

function normalizeBaseUrl(url: string): string {
  return url.replace(/\/+$/, "");
}

export async function GET() {
  const upstreamBaseUrl = normalizeBaseUrl(DASHBOARD_API_BASE_URL);
  const upstreamUrl = `${upstreamBaseUrl}/capabilities`;

  const controller = new AbortController();
  const timeoutId = setTimeout(() => controller.abort(), CAPABILITIES_TIMEOUT_MS);

  try {
    const response = await fetch(upstreamUrl, {
      headers: {
        "X-Admin-Token": DASHBOARD_ADMIN_TOKEN,
        accept: "application/json"
      },
      cache: "no-store",
      signal: controller.signal
    });

    const text = await response.text();
    return new NextResponse(text, {
      status: response.status,
      headers: {
        "cache-control": "no-store",
        "content-type": response.headers.get("content-type") ?? "application/json"
      }
    });
  } catch {
    return NextResponse.json(
      {
        status: "unknown",
        command_safety: {
          enforced: false,
          reason_required: false
        }
      },
      { status: 200 }
    );
  } finally {
    clearTimeout(timeoutId);
  }
}
