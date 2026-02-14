import { NextRequest, NextResponse } from "next/server";

const DASHBOARD_ADMIN_TOKEN = process.env.DASHBOARD_ADMIN_TOKEN ?? "test-admin-token";

const HEALTH_TIMEOUT_MS = 2000;
const BOTS_TIMEOUT_MS = 5000;

function withTimeoutSignal(timeoutMs: number) {
  const controller = new AbortController();
  const timeoutId = setTimeout(() => controller.abort(), timeoutMs);
  return { signal: controller.signal, timeoutId };
}

function normalizeBaseUrl(url: string): string {
  return url.replace(/\/+$/, "");
}

function getCandidateUpstreams(): string[] {
  const fromEnv = process.env.DASHBOARD_API_BASE_URL?.trim();
  const candidates = [
    fromEnv,
    "http://localhost:8000",
    "http://127.0.0.1:8000",
    "http://dashboard-api:8000",
    "http://host.docker.internal:8000"
  ].filter((v): v is string => Boolean(v && v.length > 0));

  return Array.from(new Set(candidates.map(normalizeBaseUrl)));
}

function errorMessage(error: unknown): string {
  if (error instanceof Error) {
    if (error.name === "AbortError") {
      return "request timed out";
    }
    return error.message;
  }
  return "unknown upstream error";
}

async function isHealthy(baseUrl: string): Promise<boolean> {
  const { signal, timeoutId } = withTimeoutSignal(HEALTH_TIMEOUT_MS);
  try {
    const response = await fetch(`${baseUrl}/healthz`, {
      method: "GET",
      cache: "no-store",
      signal
    });
    return response.ok;
  } catch {
    return false;
  } finally {
    clearTimeout(timeoutId);
  }
}

export async function GET(request: NextRequest) {
  const { searchParams } = new URL(request.url);
  const page = searchParams.get("page") ?? "1";
  const pageSize = searchParams.get("page_size") ?? "50";

  const triedUpstreams = getCandidateUpstreams();

  let selectedUpstream: string | null = null;
  for (const baseUrl of triedUpstreams) {
    if (await isHealthy(baseUrl)) {
      selectedUpstream = baseUrl;
      break;
    }
  }

  if (!selectedUpstream) {
    const message = "No reachable dashboard-api upstream (health checks failed)";
    console.error("[api/bots] bad gateway", {
      requestUrl: request.url,
      triedUpstreams,
      selectedUpstream,
      errorMessage: message
    });
    return NextResponse.json(
      {
        error: "bad_gateway",
        message,
        tried_upstreams: triedUpstreams,
        selected_upstream: selectedUpstream,
        target: selectedUpstream
      },
      { status: 502 }
    );
  }

  const upstreamUrl = `${selectedUpstream}/bots?page=${page}&page_size=${pageSize}`;
  const { signal, timeoutId } = withTimeoutSignal(BOTS_TIMEOUT_MS);

  try {
    const response = await fetch(upstreamUrl, {
      method: "GET",
      headers: {
        "X-Admin-Token": DASHBOARD_ADMIN_TOKEN
      },
      cache: "no-store",
      signal
    });

    const text = await response.text();
    return new NextResponse(text, {
      status: response.status,
      headers: {
        "content-type": response.headers.get("content-type") ?? "application/json"
      }
    });
  } catch (error) {
    const message = errorMessage(error);
    console.error("[api/bots] upstream request failed", {
      requestUrl: request.url,
      triedUpstreams,
      selectedUpstream,
      errorMessage: message,
      errorStack: error instanceof Error ? error.stack : undefined,
      errorCause: error instanceof Error ? error.cause : undefined
    });
    return NextResponse.json(
      {
        error: "bad_gateway",
        message,
        tried_upstreams: triedUpstreams,
        selected_upstream: selectedUpstream,
        target: selectedUpstream
      },
      { status: 502 }
    );
  } finally {
    clearTimeout(timeoutId);
  }
}
