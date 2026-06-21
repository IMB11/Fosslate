import { NextResponse, type NextRequest } from "next/server";

const SETUP_PATH = "/admin/setup";

let setupNotNeeded = false;
let setupCheckPromise: Promise<SetupCheckResult> | null = null;

type SetupCheckResult = "required" | "not_needed" | "unknown";

export async function middleware(request: NextRequest) {
  const { pathname } = request.nextUrl;
  const isSetupPath = pathname === SETUP_PATH;

  try {
    const setup = await checkSetup();

    if (isSetupPath && setup === "not_needed") {
      const url = request.nextUrl.clone();
      url.pathname = "/";
      url.search = "";
      return NextResponse.redirect(url);
    }

    if (!isSetupPath && setup === "required") {
      const url = request.nextUrl.clone();
      url.pathname = SETUP_PATH;
      url.search = "";
      return NextResponse.redirect(url);
    }
  } catch {
    return NextResponse.next();
  }

  return NextResponse.next();
}

export const config = {
  matcher: ["/((?!api|_next/static|_next/image|favicon.ico|.*\\..*).*)"],
};

function backendBaseUrl() {
  return (process.env.INTERNAL_API_URL ?? "http://127.0.0.1:4000").replace(
    /\/$/,
    "",
  );
}

async function checkSetup(): Promise<SetupCheckResult> {
  if (setupNotNeeded) {
    return "not_needed";
  }

  setupCheckPromise ??= checkSetupRequired().finally(() => {
    setupCheckPromise = null;
  });

  return setupCheckPromise;
}

async function checkSetupRequired(): Promise<SetupCheckResult> {
  const response = await fetch(`${backendBaseUrl()}/setup/check`, {
    cache: "no-store",
  });

  if (response.status === 404) {
    setupNotNeeded = true;
    return "not_needed";
  }

  if (response.status === 204) {
    return "required";
  }

  return "unknown";
}
