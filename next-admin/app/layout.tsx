import type { Metadata } from "next";
import "./globals.css";
import { ChakraProviders } from "../src/providers/ChakraProviders";

export const metadata: Metadata = {
  title: "Next Admin",
  description: "Admin panel migrated from Vue3 to Next.js",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="zh-CN">
      <body>
        <ChakraProviders>{children}</ChakraProviders>
      </body>
    </html>
  );
}
