import { expect, test } from "@playwright/test";
import { sendGift, startExample } from "./helpers";

test("play gift raises balance and grows the tree", async ({ page }) => {
  await startExample(page, "Leo");
  await expect(page.getByTestId("tree")).toHaveAttribute("data-stage", "0");
  await sendGift(page);
  await expect(page.getByTestId("balance")).toHaveText("$1.00");
  await expect(page.getByTestId("tree")).toHaveAttribute("data-stage", "1");
});
