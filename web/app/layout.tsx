import type { ReactNode } from "react";
import localFont from "next/font/local";
import Link from "next/link";
import "./globals.css";

const fraunces = localFont({
  src: "./fonts/fraunces-latin.woff2",
  variable: "--font-fraunces",
  weight: "400 800",
});

const inter = localFont({
  src: "./fonts/inter-latin.woff2",
  variable: "--font-inter",
  weight: "400 700",
});

export const metadata = {
  title: "Birthday Vault",
  description:
    "A gift for a child that relatives can add to, and nobody can open until the child turns 18.",
};

export default function RootLayout({ children }: { children: ReactNode }) {
  return (
    <html lang="en" className={`${fraunces.variable} ${inter.variable}`}>
      <body className="min-h-screen flex flex-col">
        <header className="px-4 sm:px-8 py-4 flex items-center justify-between max-w-5xl w-full mx-auto">
          <Link href="/" className="font-display text-xl font-semibold text-leaf-dark">
            Birthday Vault
          </Link>
          <span className="text-xs sm:text-sm text-muted">Built on NEAR</span>
        </header>
        <main className="flex-1 w-full max-w-5xl mx-auto px-4 sm:px-8">{children}</main>
        <footer className="mt-16 border-t border-line">
          <div className="max-w-5xl mx-auto px-4 sm:px-8 py-6 flex flex-wrap gap-x-6 gap-y-2 text-sm text-muted">
            <Link href="/limits/" className="underline underline-offset-4 hover:text-ink">
              Limits
            </Link>
            <span>Demo on NEAR testnet with play money. Unaudited code.</span>
          </div>
        </footer>
      </body>
    </html>
  );
}
