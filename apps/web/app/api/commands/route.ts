import { NextRequest, NextResponse } from "next/server";

const DASHBOARD_API_BASE_URL =
  process.env.DASHBOARD_API_BASE_URL ?? process.env.DASHBOARD_API_URL ?? "http://localhost:8000";
const DASHBOARD_ADMIN_TOKEN = process.env.DASHBOARD_ADMIN_TOKEN ?? process.env.ADMIN_TOKEN ?? "test-admin-token";
const COMMANDS_TIMEOUT_MS = 5000;

function normalizeBaseUrl(url: string): string {
  return url.replace(/\/+$/, "");
}

function toErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    if (error.name === "AbortError") {
      return "request timed out while contacting commands backend";
    }
    return error.message;
  }
  return "failed to reach commands backend";
}

async function proxyRequest(request: NextRequest, method: "GET" | "POST") {
  const upstreamBaseUrl = normalizeBaseUrl(DASHBOARD_API_BASE_URL);
  const search = new URL(request.url).searchParams.toString();
  const upstreamUrl = `${upstreamBaseUrl}/commands${search ? `?${search}` : ""}`;

  const controller = new AbortController();
  const timeoutId = setTimeout(() => controller.abort(), COMMANDS_TIMEOUT_MS);

  try {
    const body = method === "POST" ? await request.text() : undefined;

    const response = await fetch(upstreamUrl, {
      method,
      headers: {
        "X-Admin-Token": DASHBOARD_ADMIN_TOKEN,
        ...(method === "POST" ? { "content-type": "application/json" } : {})
      },
      body,
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
        code: "COMMANDS_PROXY_UNAVAILABLE",
        message,
        details: {
          request_url: request.url,
          upstream: upstreamBaseUrl
        }
      },
      { status: 502 }
    );
  } finally {
    clearTimeout(timeoutId);
  }
}

export async function GET(request: NextRequest) {
  return proxyRequest(request, "GET");
}

export async function POST(request: NextRequest) {
  return proxyRequest(request, "POST");
}
