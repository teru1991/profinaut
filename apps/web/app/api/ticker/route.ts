import { NextRequest, NextResponse } from "next/server";

const MARKETDATA_API_BASE_URL =
  process.env.MARKETDATA_API_BASE_URL ?? process.env.MARKETDATA_BASE_URL ?? "http://127.0.0.1:8081";

const ALLOWED_EXCHANGES = new Set(["gmo", "binance"]);
const SYMBOL_PATTERN = /^[A-Z0-9_/:.-]{3,32}$/;

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
  const params = new URL(request.url).searchParams;
  const exchange = (params.get("exchange") ?? "gmo").trim().toLowerCase();
  const symbol = (params.get("symbol") ?? "BTC_JPY").trim();

  if (!ALLOWED_EXCHANGES.has(exchange)) {
    return NextResponse.json(
      {
        error: "invalid_exchange",
        message: `unsupported exchange '${exchange}'`
      },
      { status: 400, headers: { "cache-control": "no-store" } }
    );
  }

  if (!SYMBOL_PATTERN.test(symbol)) {
    return NextResponse.json(
      {
        error: "invalid_symbol",
        message: "symbol format is invalid"
      },
      { status: 400, headers: { "cache-control": "no-store" } }
    );
  }

  const upstreamBaseUrl = normalizeBaseUrl(MARKETDATA_API_BASE_URL);
  const upstreamUrl = `${upstreamBaseUrl}/ticker/latest?exchange=${encodeURIComponent(exchange)}&symbol=${encodeURIComponent(symbol)}`;

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
