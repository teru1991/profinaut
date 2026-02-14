import { randomUUID } from "crypto";

import { NextRequest, NextResponse } from "next/server";

const MARKETDATA_API_BASE_URL =
  process.env.MARKETDATA_API_BASE_URL ?? process.env.MARKETDATA_BASE_URL ?? "http://127.0.0.1:8081";

const DEFAULT_EXCHANGE = "gmo";
const DEFAULT_SYMBOL = "BTC_JPY";
const ALLOWED_EXCHANGES = new Set(["gmo", "binance"]);
const SYMBOL_PATTERN = /^[A-Z0-9_/:.-]{3,32}$/;

type ErrorPayload = {
  error: {
    code: string;
    message: string;
  };
  request_id: string;
};

function normalizeBaseUrl(url: string): string {
  return url.replace(/\/+$/, "");
}

function resolveRequestId(request: NextRequest): string {
  const reqId = request.headers.get("x-request-id")?.trim();
  return reqId && reqId.length > 0 ? reqId : randomUUID();
}

function errorResponse(
  requestId: string,
  status: number,
  code: string,
  message: string
): NextResponse<ErrorPayload> {
  return NextResponse.json(
    {
      error: { code, message },
      request_id: requestId
    },
    {
      status,
      headers: {
        "cache-control": "no-store",
        "x-request-id": requestId
      }
    }
  );
}

async function upstreamErrorMessage(response: Response): Promise<string> {
  const contentType = response.headers.get("content-type") ?? "";
  if (contentType.includes("application/json")) {
    try {
      const payload = await response.json();
      if (payload && typeof payload === "object") {
        const message = (payload as Record<string, unknown>).message;
        if (typeof message === "string" && message.trim()) {
          return message;
        }
      }
    } catch {
      // ignore parse failures
    }
  }
  return `marketdata request failed with status ${response.status}`;
}

export async function GET(request: NextRequest) {
  const requestId = resolveRequestId(request);
  const params = new URL(request.url).searchParams;
  const exchange = (params.get("exchange") ?? DEFAULT_EXCHANGE).trim().toLowerCase();
  const symbol = (params.get("symbol") ?? DEFAULT_SYMBOL).trim().toUpperCase();

  if (!ALLOWED_EXCHANGES.has(exchange)) {
    return errorResponse(requestId, 400, "INVALID_EXCHANGE", `Unsupported exchange '${exchange}'.`);
  }

  if (!SYMBOL_PATTERN.test(symbol)) {
    return errorResponse(requestId, 400, "INVALID_SYMBOL", "Symbol format is invalid.");
  }

  const upstreamBaseUrl = normalizeBaseUrl(MARKETDATA_API_BASE_URL);
  const upstreamUrl = `${upstreamBaseUrl}/ticker/latest?exchange=${encodeURIComponent(exchange)}&symbol=${encodeURIComponent(symbol)}`;

  try {
    const response = await fetch(upstreamUrl, { cache: "no-store" });

    if (!response.ok) {
      const message = await upstreamErrorMessage(response);
      return errorResponse(requestId, 502, "UPSTREAM_ERROR", message);
    }

    const body = await response.text();
    return new NextResponse(body, {
      status: 200,
      headers: {
        "cache-control": "no-store",
        "content-type": response.headers.get("content-type") ?? "application/json",
        "x-request-id": requestId
      }
    });
  } catch {
    return errorResponse(
      requestId,
      502,
      "UPSTREAM_UNAVAILABLE",
      "Failed to reach marketdata service."
    );
  }
}
