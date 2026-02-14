import { NextRequest, NextResponse } from "next/server";

const DASHBOARD_API_BASE_URL = process.env.DASHBOARD_API_BASE_URL ?? "http://localhost:8000";
const DASHBOARD_ADMIN_TOKEN = process.env.DASHBOARD_ADMIN_TOKEN ?? "test-admin-token";

export async function GET(request: NextRequest) {
  const { searchParams } = new URL(request.url);
  const page = searchParams.get("page") ?? "1";
  const pageSize = searchParams.get("page_size") ?? "50";

  try {
    const response = await fetch(`${DASHBOARD_API_BASE_URL}/bots?page=${page}&page_size=${pageSize}`, {
      headers: {
        "X-Admin-Token": DASHBOARD_ADMIN_TOKEN
      },
      cache: "no-store"
    });

    const text = await response.text();
    return new NextResponse(text, {
      status: response.status,
      headers: {
        "content-type": response.headers.get("content-type") ?? "application/json"
      }
    });
  } catch (error) {
    const message = error instanceof Error ? error.message : "failed to reach dashboard-api";
    return NextResponse.json(
      {
        error: "bad_gateway",
        message
      },
      { status: 502 }
    );
  }
}
