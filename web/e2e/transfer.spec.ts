import { expect, test } from "@playwright/test";
import { sendGift, startExample, waitForUnlock } from "./helpers";

test("new owner withdraws and old owner is refused", async ({ page }) => {
  await startExample(page, "Noah");
  await sendGift(page);
  await sendGift(page);
  await expect(page.getByTestId("balance")).toHaveText("$2.00");
  await expect(page.getByTestId("grew")).toBeVisible();
  await page.getByRole("button", { name: "Try to withdraw early" }).click();
  await expect(page.getByTestId("message")).toHaveText("Refused by the contract: the vault is still locked.");
  await waitForUnlock(page);
  // The lock has ended, so the "still locked" refusal and the gift badge must not stay on screen.
  await expect(page.getByTestId("message")).toHaveCount(0);
  await expect(page.getByTestId("grew")).toHaveCount(0);
  await page.getByRole("button", { name: "Transfer it" }).click();
  await expect(page.getByTestId("owner")).toHaveText("the new owner");
  await page.getByRole("button", { name: "Old owner tries to withdraw" }).click();
  await expect(page.getByTestId("message")).toHaveText("Refused by the contract: only the current owner can withdraw.");
  await expect(page.getByTestId("balance")).toHaveText("$2.00");
  await page.getByRole("button", { name: "New owner withdraws" }).click();
  await expect(page.getByTestId("balance")).toHaveText("$0.00");
  await expect(page.getByTestId("done")).toContainText("The money went to the new owner");
});
