import { NextRequest, NextResponse } from "next/server";

const DASHBOARD_API_BASE_URL =
  process.env.DASHBOARD_API_BASE_URL ?? process.env.DASHBOARD_API_URL ?? "http://localhost:8000";
const DASHBOARD_ADMIN_TOKEN = process.env.DASHBOARD_ADMIN_TOKEN ?? process.env.ADMIN_TOKEN ?? "test-admin-token";
const BOTS_TIMEOUT_MS = 5000;

function normalizeBaseUrl(url: string): string {
  return url.replace(/\/+$/, "");
}

function toErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    if (error.name === "AbortError") {
      return "request timed out while contacting bots backend";
    }
    return error.message;
  }
  return "failed to reach bots backend";
}

export async function GET(request: NextRequest) {
  const { searchParams } = new URL(request.url);
  const page = searchParams.get("page") ?? "1";
  const pageSize = searchParams.get("page_size") ?? "50";

  const upstreamBaseUrl = normalizeBaseUrl(DASHBOARD_API_BASE_URL);
  const upstreamUrl = `${upstreamBaseUrl}/bots?page=${page}&page_size=${pageSize}`;

  const controller = new AbortController();
  const timeoutId = setTimeout(() => controller.abort(), BOTS_TIMEOUT_MS);

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
    const message = `${toErrorMessage(error)}. Set DASHBOARD_API_BASE_URL (or DASHBOARD_API_URL) to your dashboard-api URL and ensure backend is running.`;
    console.error("[api/bots] proxy failed", {
      requestUrl: request.url,
      upstream: upstreamBaseUrl,
      message
    });
    return NextResponse.json(
      {
        error: "bad_gateway",
        message,
        requestUrl: request.url,
        upstream: upstreamBaseUrl
      },
      { status: 502 }
    );
  } finally {
    clearTimeout(timeoutId);
  }
}
