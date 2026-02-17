import type { Metadata } from "next";
import type { ReactNode } from "react";

import { ThemeProvider } from "../components/ThemeProvider";
import { NavShell } from "../components/NavShell";
import { StatusRibbon } from "../components/StatusRibbon";

import "./globals.css";

export const metadata: Metadata = {
  title: "Profinaut Dashboard",
  description: "Multi-Exchange / Multi-Language Bot Management Dashboard"
};

export default function RootLayout({ children }: { children: ReactNode }) {
  return (
    <html lang="en" data-theme="dark" suppressHydrationWarning>
      <body>
        <ThemeProvider>
          <StatusRibbon />
          <NavShell>{children}</NavShell>
        </ThemeProvider>
      </body>
    </html>
  );
}
