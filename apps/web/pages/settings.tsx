import type { GetServerSideProps } from "next";

export const getServerSideProps = (async () => ({
  redirect: {
    destination: "/settings/profile",
    permanent: false,
  },
})) satisfies GetServerSideProps;

export default function SettingsIndexPage() {
  return null;
}
