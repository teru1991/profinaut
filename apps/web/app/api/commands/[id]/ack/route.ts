import { NextRequest, NextResponse } from "next/server";

const DASHBOARD_API_BASE_URL =
  process.env.DASHBOARD_API_BASE_URL ?? process.env.DASHBOARD_API_URL ?? "http://localhost:8000";
const DASHBOARD_ADMIN_TOKEN = process.env.DASHBOARD_ADMIN_TOKEN ?? process.env.ADMIN_TOKEN ?? "test-admin-token";
const ACK_TIMEOUT_MS = 5000;

function normalizeBaseUrl(url: string): string {
  return url.replace(/\/+$/, "");
}

function toErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    if (error.name === "AbortError") {
      return "request timed out while contacting command ack backend";
    }
    return error.message;
  }
  return "failed to reach command ack backend";
}

export async function POST(request: NextRequest, context: { params: { id: string } }) {
  const commandId = context.params.id;
  const upstreamBaseUrl = normalizeBaseUrl(DASHBOARD_API_BASE_URL);
  const upstreamUrl = `${upstreamBaseUrl}/commands/${encodeURIComponent(commandId)}/ack`;

  const controller = new AbortController();
  const timeoutId = setTimeout(() => controller.abort(), ACK_TIMEOUT_MS);

  try {
    const body = await request.text();
    const response = await fetch(upstreamUrl, {
      method: "POST",
      headers: {
        "X-Admin-Token": DASHBOARD_ADMIN_TOKEN,
        "content-type": "application/json"
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
        code: "COMMAND_ACK_PROXY_UNAVAILABLE",
        message,
        details: {
          request_url: request.url,
          upstream: upstreamBaseUrl,
          command_id: commandId
        }
      },
      { status: 502 }
    );
  } finally {
    clearTimeout(timeoutId);
  }
}
