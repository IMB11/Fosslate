import NextApp, { type AppContext, type AppProps } from "next/app";
import { Archivo_Black, Space_Grotesk } from "next/font/google";
import Head from "next/head";
import { useRouter } from "next/router";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useState } from "react";

import { Navbar } from "@/components/layout/Navbar";
import type { AuthUser } from "@/lib/auth-client";
import { getServerAuthSession } from "@/lib/auth-server";

import "./globals.css";

const archivoBlack = Archivo_Black({
  subsets: ["latin"],
  weight: "400",
  variable: "--font-head",
  display: "swap",
});

const space = Space_Grotesk({
  subsets: ["latin"],
  weight: "400",
  variable: "--font-sans",
  display: "swap",
});

type FosslatePageProps = {
  initialAuthUser?: AuthUser | null;
};

type FosslateAppProps = AppProps<FosslatePageProps>;

export default function App({ Component, pageProps }: FosslateAppProps) {
  const router = useRouter();
  const [queryClient] = useState(
    () =>
      new QueryClient({
        defaultOptions: {
          queries: {
            refetchOnWindowFocus: false,
            retry: false,
          },
          mutations: {
            retry: false,
          },
        },
      }),
  );
  const showNavbar = !standaloneRoute(router.pathname);

  return (
    <QueryClientProvider client={queryClient}>
      <Head>
        <title>Fosslate</title>
        <meta name="description" content="Self-hosted localisation platform" />
      </Head>
      <div className={`${archivoBlack.variable} ${space.variable}`}>
        {showNavbar ? (
          <div className="min-h-screen bg-background text-foreground">
            <Navbar initialUser={pageProps.initialAuthUser} />
            <Component {...pageProps} />
          </div>
        ) : (
          <Component {...pageProps} />
        )}
      </div>
    </QueryClientProvider>
  );
}

App.getInitialProps = async (appContext: AppContext) => {
  const appProps = await NextApp.getInitialProps(appContext);

  if (standaloneRoute(appContext.ctx.pathname)) {
    return appProps;
  }

  const session = await getServerAuthSession(
    appContext.ctx.req,
    appContext.ctx.res,
  );

  return {
    ...appProps,
    pageProps: {
      ...appProps.pageProps,
      ...(session.resolved ? { initialAuthUser: session.user } : {}),
    },
  };
};

function standaloneRoute(pathname: string): boolean {
  return [
    "/admin/setup",
    "/forgot-password",
    "/login",
    "/reset-password",
    "/signup",
  ].includes(pathname);
}
