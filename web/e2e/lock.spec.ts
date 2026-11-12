import { expect, test } from "@playwright/test";
import { sendGift, startExample } from "./helpers";

test("early withdrawal is refused", async ({ page }) => {
  await startExample(page, "Ava");
  await sendGift(page);
  await page.getByRole("button", { name: "Try to withdraw early" }).click();
  await expect(page.getByTestId("message")).toHaveText("Refused by the contract: the vault is still locked.");
  await expect(page.getByTestId("balance")).toHaveText("$1.00");
});

test("unknown vault id is refused", async ({ page }) => {
  const coll = process.env.E2E_COLLECTION_ID!;
  for (const id of [`bv-999999.${coll}`, "bv-1.someone-else.testnet", "bv-1.g1.alice.near", "not-a-vault"]) {
    await page.goto(`/vault/?id=${encodeURIComponent(id)}`);
    await expect(page.getByTestId("vault-missing")).toBeVisible();
    await expect(page.getByTestId("actions")).toHaveCount(0);
  }
  await page.goto(`/gift/?id=${encodeURIComponent(`bv-999999.${coll}`)}`);
  await expect(page.getByTestId("vault-missing")).toBeVisible();
});
