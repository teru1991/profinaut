import { NextRequest, NextResponse } from "next/server";

const DASHBOARD_API_BASE_URL =
  process.env.DASHBOARD_API_BASE_URL ?? process.env.DASHBOARD_API_URL ?? "http://localhost:8000";
const MARKET_PROXY_TIMEOUT_MS = 2500;

function normalizeBaseUrl(url: string): string {
  return url.replace(/\/+$/, "");
}

function toErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    if (error.name === "AbortError") {
      return "request timed out while contacting dashboard-api marketdata proxy";
    }
    return error.message;
  }
  return "failed to reach dashboard-api marketdata proxy";
}

export async function GET(request: NextRequest) {
  const search = new URL(request.url).searchParams.toString();
  const upstreamBaseUrl = normalizeBaseUrl(DASHBOARD_API_BASE_URL);
  const upstreamUrl = `${upstreamBaseUrl}/api/markets/ticker/latest${search ? `?${search}` : ""}`;

  const controller = new AbortController();
  const timeoutId = setTimeout(() => controller.abort(), MARKET_PROXY_TIMEOUT_MS);

  try {
    const response = await fetch(upstreamUrl, {
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
  } catch (error) {
    const message = `${toErrorMessage(error)}. Set DASHBOARD_API_BASE_URL (or DASHBOARD_API_URL) to dashboard-api URL.`;
    return NextResponse.json(
      {
        code: "MARKETS_PROXY_UNAVAILABLE",
        message,
        details: {
          request_url: request.url,
          upstream: upstreamBaseUrl
        }
      },
      { status: 502, headers: { "cache-control": "no-store" } }
    );
  } finally {
    clearTimeout(timeoutId);
  }
}
