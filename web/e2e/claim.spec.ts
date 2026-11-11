import { expect, test } from "@playwright/test";
import { sendGift, startExample, waitForUnlock } from "./helpers";

test("child claims when guardian key is lost", async ({ page }) => {
  await startExample(page, "Zoe");
  await sendGift(page);
  await waitForUnlock(page);
  await page.getByRole("button", { name: "The guardian lost their key" }).click();
  await expect(page.getByTestId("owner")).toHaveText("Zoe (the child)");
  await page.getByRole("button", { name: "Zoe withdraws" }).click();
  await expect(page.getByTestId("balance")).toHaveText("$0.00");
  await expect(page.getByTestId("done")).toContainText("The money went to Zoe (the child)");
});
