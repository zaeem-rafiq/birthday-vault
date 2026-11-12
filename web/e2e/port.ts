// The port the end-to-end tests serve the site on. It comes from this checkout's path, so two
// worktrees testing at the same time use different ports, and the same checkout always uses the
// same one (./check makes runs in one checkout take turns).
import { createHash } from "node:crypto";
import path from "node:path";

const web = path.resolve(__dirname, "..");
export const E2E_PORT = 4200 + (createHash("sha256").update(web).digest().readUInt16BE(0) % 700);
