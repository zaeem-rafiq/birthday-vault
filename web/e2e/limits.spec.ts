import { expect, test } from "@playwright/test";

test("limits page lists the known limits", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("link", { name: "Limits" }).click();
  await expect(page.getByRole("heading", { name: "Limits", level: 1 })).toBeVisible();
  const list = page.getByTestId("limits");
  for (const text of [
    "It earns nothing.",
    "It depends on this contract and this network lasting.",
    "USDC's issuer can freeze an account.",
    "The code is unaudited.",
    "This demo uses play money and short timers.",
    "A wrong child account cannot be corrected.",
    "The guardian can take the money on the lock date.",
  ]) {
    await expect(list.getByRole("heading", { name: text })).toBeVisible();
  }
});
