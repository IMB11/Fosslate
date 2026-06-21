import type { AppProps } from "next/app";
import { Archivo_Black, Space_Grotesk } from "next/font/google";
import Head from "next/head";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useState } from "react";

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

export default function App({ Component, pageProps }: AppProps) {
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

  return (
    <QueryClientProvider client={queryClient}>
      <Head>
        <title>Fosslate</title>
        <meta name="description" content="Self-hosted localisation platform" />
      </Head>
      <div className={`${archivoBlack.variable} ${space.variable}`}>
        <Component {...pageProps} />
      </div>
    </QueryClientProvider>
  );
}

