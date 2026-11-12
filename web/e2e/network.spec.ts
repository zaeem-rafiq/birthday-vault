import { expect, test } from "@playwright/test";
import { startExample } from "./helpers";

test("vault page explains an unreachable network and recovers", async ({ page }) => {
  const id = await startExample(page, "Ruby");
  // Cut the page off from the chain: every RPC call fails until the route is removed.
  const rpc = process.env.E2E_RPC_URL!.replace(/\/$/, "");
  await page.route(`${rpc}/**`, (route) => route.abort());
  await page.route(rpc, (route) => route.abort());
  await page.reload();
  await expect(page.getByTestId("vault-unreachable")).toContainText("The test network did not answer");
  await page.unrouteAll();
  await page.getByRole("button", { name: "Try again" }).click();
  await expect(page.getByRole("heading", { name: "Ruby's vault" })).toBeVisible();
  expect(new URL(page.url()).searchParams.get("id")).toBe(id);
});
