import { Html, Head, Main, NextScript } from "next/document";

const themeScript = `
(function () {
  try {
    var darkMode = window.localStorage.getItem("darkMode");
    var prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
    if (darkMode === "dark" || (darkMode !== "light" && prefersDark)) {
      document.documentElement.classList.add("dark");
    } else {
      document.documentElement.classList.remove("dark");
    }
  } catch (_) {}
})();
`;

export default function Document() {
  return (
    <Html lang="en">
      <Head>
        <script dangerouslySetInnerHTML={{ __html: themeScript }} />
      </Head>
      <body>
        <Main />
        <NextScript />
      </body>
    </Html>
  );
}
