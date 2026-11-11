// Starts a local NEAR sandbox with the real contracts, builds the site against it, and serves
// the build. Only 1Click is stubbed, in the tests themselves; the chain is never stubbed.
// The sandbox launcher is tests/examples/e2e_sandbox.rs; its hub keys exist only in that sandbox.
import { spawn, execFileSync, type ChildProcess } from "node:child_process";
import { createInterface } from "node:readline";
import net from "node:net";
import path from "node:path";

export const PORT = 4173;
export const LOCK_SECONDS = 25;
const web = path.resolve(__dirname, "..");
const repo = path.resolve(web, "..");

function startSandbox(): Promise<{ proc: ChildProcess; env: Record<string, string> }> {
  return new Promise((resolve, reject) => {
    const proc = spawn("cargo", ["run", "-q", "--locked", "-p", "tests", "--example", "e2e_sandbox"], {
      cwd: path.join(repo, "tests"),
      stdio: ["pipe", "pipe", "inherit"],
    });
    const timer = setTimeout(() => reject(new Error("The sandbox did not start in 5 minutes")), 300_000);
    proc.on("exit", (code) => reject(new Error(`The sandbox launcher exited with ${code}`)));
    createInterface({ input: proc.stdout! }).on("line", (line) => {
      if (!line.startsWith("E2E_READY ")) return;
      clearTimeout(timer);
      const r = JSON.parse(line.slice("E2E_READY ".length));
      resolve({
        proc,
        env: {
          NEXT_PUBLIC_NETWORK: "sandbox",
          NEXT_PUBLIC_DEMO_RPC_URLS: r.rpc,
          NEXT_PUBLIC_COLLECTION_ID: r.collection,
          NEXT_PUBLIC_HUB_ID: r.hub,
          NEXT_PUBLIC_USDC_ID: r.usdc,
          NEXT_PUBLIC_HUB_KEYS: r.hubKeys,
          NEXT_PUBLIC_LOCK_SECONDS: String(LOCK_SECONDS),
          NEXT_PUBLIC_EXPLORER_URL: "http://127.0.0.1",
        },
      });
    });
  });
}

/** Fails fast when another end-to-end run already serves on the port, instead of testing its build. */
function assertPortFree(port: number): Promise<void> {
  return new Promise((resolve, reject) => {
    const probe = net.createServer();
    probe.once("error", () =>
      reject(new Error(`Port ${port} is in use: another end-to-end run is active. ./check waits for it; run one at a time.`)),
    );
    probe.once("listening", () => probe.close(() => resolve()));
    probe.listen(port, "127.0.0.1");
  });
}

function startServer(): Promise<ChildProcess> {
  return new Promise((resolve, reject) => {
    const server = spawn(process.execPath, [path.join(web, "e2e/serve.mjs"), path.join(web, "out"), String(PORT)], {
      stdio: ["ignore", "pipe", "inherit"],
    });
    server.on("exit", (code) => reject(new Error(`The test web server exited with ${code}`)));
    createInterface({ input: server.stdout! }).on("line", (line) => {
      if (line.startsWith("SERVE_READY")) {
        server.removeAllListeners("exit");
        resolve(server);
      }
    });
  });
}

export default async function globalSetup() {
  await assertPortFree(PORT);
  const { proc, env } = await startSandbox();
  // The build writes `out/`, replacing any earlier build there.
  execFileSync("npx", ["next", "build"], {
    cwd: web,
    stdio: "inherit",
    env: { ...process.env, ...env, NEXT_TELEMETRY_DISABLED: "1" },
  });
  const server = await startServer();
  process.env.E2E_COLLECTION_ID = env.NEXT_PUBLIC_COLLECTION_ID;
  return async () => {
    server.kill();
    proc.removeAllListeners("exit");
    proc.stdin?.end();
    await new Promise((r) => setTimeout(r, 1500));
    proc.kill();
  };
}
