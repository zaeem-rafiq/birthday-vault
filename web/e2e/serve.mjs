// A tiny static server for the exported site in `out/`, used only by the end-to-end tests.
// Usage: node e2e/serve.mjs <folder> <port>
import { createServer } from "node:http";
import { readFile, stat } from "node:fs/promises";
import { extname, join, normalize } from "node:path";

const [root, port] = [process.argv[2], Number(process.argv[3])];
const types = { ".html": "text/html", ".js": "text/javascript", ".css": "text/css", ".woff2": "font/woff2", ".svg": "image/svg+xml", ".json": "application/json", ".txt": "text/plain", ".ico": "image/x-icon" };

createServer(async (req, res) => {
  let path = normalize(decodeURIComponent(new URL(req.url, "http://x").pathname)).replace(/^(\.\.[/\\])+/, "");
  let file = join(root, path);
  try {
    if ((await stat(file)).isDirectory()) file = join(file, "index.html");
    const body = await readFile(file);
    res.writeHead(200, { "Content-Type": types[extname(file)] || "application/octet-stream" });
    res.end(body);
  } catch {
    res.writeHead(404, { "Content-Type": "text/html" });
    res.end(await readFile(join(root, "404.html")).catch(() => "Not found"));
  }
})
  .on("error", (e) => {
    console.error(`serve.mjs: cannot serve on port ${port}: ${e.message}`);
    process.exit(1);
  })
  .listen(port, "127.0.0.1", () => console.log(`SERVE_READY ${port}`));
