import { NextRequest, NextResponse } from "next/server";

const DASHBOARD_API_BASE_URL =
  process.env.DASHBOARD_API_BASE_URL ?? process.env.DASHBOARD_API_URL ?? "http://localhost:8000";
const DASHBOARD_ADMIN_TOKEN = process.env.DASHBOARD_ADMIN_TOKEN ?? process.env.ADMIN_TOKEN ?? "test-admin-token";

function normalizeBaseUrl(url: string): string {
  return url.replace(/\/+$/, "");
}

function toErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }
  return "failed to reach dashboard-api";
}

export async function GET(request: NextRequest) {
  const upstreamBaseUrl = normalizeBaseUrl(DASHBOARD_API_BASE_URL);
  const search = new URL(request.url).searchParams.toString();
  const upstreamUrl = `${upstreamBaseUrl}/api/markets/ticker/latest${search ? `?${search}` : ""}`;

  try {
    const response = await fetch(upstreamUrl, {
      headers: {
        "X-Admin-Token": DASHBOARD_ADMIN_TOKEN
      },
      cache: "no-store"
    });

    return new NextResponse(response.body, {
      status: response.status,
      headers: {
        "cache-control": "no-store",
        "content-type": response.headers.get("content-type") ?? "application/json"
      }
    });
  } catch (error) {
    return NextResponse.json(
      {
        error: "bad_gateway",
        message: toErrorMessage(error)
      },
      { status: 502, headers: { "cache-control": "no-store" } }
    );
  }
}
