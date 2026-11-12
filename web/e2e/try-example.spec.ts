import { expect, test } from "@playwright/test";
import { startExample } from "./helpers";

test("visitor creates a vault with no login", async ({ page }) => {
  const id = await startExample(page, "Maya");
  await expect(page.getByRole("heading", { name: "Maya's vault" })).toBeVisible();
  await expect(page.getByText("TESTNET, PLAY MONEY")).toBeVisible();
  await expect(page.getByTestId("keys")).toHaveText("0");
  await expect(page.getByTestId("owner")).toHaveText("the parent (guardian)");
  await expect(page.getByTestId("lock")).toHaveText(/^\d:\d\d$/);
  await expect(page.getByTestId("tree")).toHaveAttribute("data-stage", "0");
  // A reload resumes the same demo from this browser's storage.
  await page.reload();
  await expect(page.getByRole("heading", { name: "Maya's vault" })).toBeVisible();
  expect(new URL(page.url()).searchParams.get("id")).toBe(id);
});
