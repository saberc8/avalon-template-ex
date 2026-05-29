import type { Metadata } from "next";
import { ThemeProvider } from "@/components/theme/theme-provider";
import { Toaster } from "@/components/ui/sonner";
import { appearanceAttributes, DEFAULT_APPEARANCE } from "@/lib/theme";
import "./globals.css";

export const metadata: Metadata = {
  title: "Avalon Admin",
  description: "Avalon admin console"
};

export default function RootLayout({
  children
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="zh-CN" suppressHydrationWarning {...appearanceAttributes(DEFAULT_APPEARANCE)}>
      <body>
        <ThemeProvider>
          {children}
          <Toaster />
        </ThemeProvider>
      </body>
    </html>
  );
}
