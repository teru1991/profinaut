import type { Metadata } from "next";
import type { ReactNode } from "react";

import { NavShell } from "../components/NavShell";

import "./globals.css";

export const metadata: Metadata = {
  title: "Profinaut Dashboard",
  description: "Multi-Exchange / Multi-Language Bot Management Dashboard"
};

export default function RootLayout({ children }: { children: ReactNode }) {
  return (
    <html lang="en">
      <body>
        <NavShell>{children}</NavShell>
      </body>
    </html>
  );
}
