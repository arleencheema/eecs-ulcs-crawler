import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "UMich EECS Course Search",
  description: "Search upper-level CS courses by topic, workload, or keyword",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en" className="h-full">
      <body className="min-h-full">{children}</body>
    </html>
  );
}