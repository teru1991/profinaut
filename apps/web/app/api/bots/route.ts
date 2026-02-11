import { NextRequest, NextResponse } from "next/server";

const API_URL = process.env.DASHBOARD_API_URL ?? "http://localhost:8000";
const ADMIN_TOKEN = process.env.ADMIN_TOKEN ?? "";

export async function GET(request: NextRequest) {
  const { searchParams } = new URL(request.url);
  const page = searchParams.get("page") ?? "1";
  const pageSize = searchParams.get("page_size") ?? "50";

  const response = await fetch(`${API_URL}/bots?page=${page}&page_size=${pageSize}`, {
    headers: {
      "X-Admin-Token": ADMIN_TOKEN
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
}
