import type { NextConfig } from "next";

// A static export: Render serves the files in `out/` with no server.
const config: NextConfig = {
  output: "export",
  images: { unoptimized: true },
  trailingSlash: true,
};

export default config;
