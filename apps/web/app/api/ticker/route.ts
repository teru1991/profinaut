import { NextRequest, NextResponse } from "next/server";

const MARKETDATA_API_BASE_URL =
  process.env.MARKETDATA_API_BASE_URL ?? process.env.MARKETDATA_BASE_URL ?? "http://127.0.0.1:8081";

function normalizeBaseUrl(url: string): string {
  return url.replace(/\/+$/, "");
}

function toErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }
  return "failed to reach marketdata service";
}

export async function GET(request: NextRequest) {
  const upstreamBaseUrl = normalizeBaseUrl(MARKETDATA_API_BASE_URL);
  const search = new URL(request.url).searchParams.toString();
  const upstreamUrl = `${upstreamBaseUrl}/ticker/latest${search ? `?${search}` : ""}`;

  try {
    const response = await fetch(upstreamUrl, { cache: "no-store" });
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
