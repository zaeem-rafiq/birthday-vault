import { expect, type Page } from "@playwright/test";

/** Runs "Try an example" from the home page and waits for the new vault page. */
export async function startExample(page: Page, nickname: string) {
  await page.goto("/");
  await page.getByLabel("Child's nickname").fill(nickname);
  await page.getByRole("button", { name: "Try an example" }).click();
  await expect(page).toHaveURL(/\/vault\/\?id=bv-\d+\./, { timeout: 120_000 });
  await expect(page.getByTestId("balance")).toHaveText("$0.00");
  return new URL(page.url()).searchParams.get("id")!;
}

export async function sendGift(page: Page) {
  await page.getByRole("button", { name: "Send a $1 play gift" }).click();
  await expect(page.getByTestId("message")).toContainText("Play gift of $1.00 received");
}

/** Waits until the vault's lock has ended. */
export async function waitForUnlock(page: Page) {
  await expect(page.getByTestId("lock")).toHaveText("Open", { timeout: 90_000 });
}
