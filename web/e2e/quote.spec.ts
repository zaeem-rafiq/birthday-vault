import { expect, test, type Page } from "@playwright/test";
import { startExample } from "./helpers";

// Only 1Click is stubbed. Every request body the page sends to it is recorded.
async function stubOneClick(page: Page) {
  const bodies: Record<string, unknown>[] = [];
  await page.route("https://1click.chaindefuser.com/**", async (route) => {
    const body = route.request().postDataJSON() ?? {};
    bodies.push(body);
    await route.fulfill({
      json: {
        quote: {
          amountInFormatted: "5.0",
          amountOutFormatted: "4.99",
          minAmountOut: "4949348",
          timeEstimate: 35,
        },
        quoteRequest: body,
      },
    });
  });
  return bodies;
}

test("gift page shows a preview quote", async ({ page }) => {
  const bodies = await stubOneClick(page);
  const id = await startExample(page, "Ivy");
  await page.goto(`/gift/?id=${encodeURIComponent(id)}`);
  await expect(page.getByTestId("preview-label")).toHaveText("Preview only. Nothing is sent.");
  await page.getByLabel(/address, for refunds/).fill("0x1111111111111111111111111111111111111111");
  await expect(page.getByTestId("amount-out")).toHaveText("4.99 USDC");
  expect(bodies.length).toBeGreaterThan(0);
  for (const b of bodies) {
    expect(b.dry).toBe(true);
    expect(b.recipient).toBe("wrap.near");
    expect(JSON.stringify(b)).not.toContain(id);
    expect(JSON.stringify(b)).not.toMatch(/\.test\.near|\.testnet|\.sandbox/);
  }
});

test("deposit address control is hidden", async ({ page }) => {
  const bodies = await stubOneClick(page);
  const id = await startExample(page, "Eli");
  await page.goto(`/gift/?id=${encodeURIComponent(id)}`);
  await page.getByLabel(/address, for refunds/).fill("0x1111111111111111111111111111111111111111");
  await expect(page.getByTestId("amount-out")).toBeVisible();
  await expect(page.getByRole("button", { name: /deposit address/i })).toHaveCount(0);
  await expect(page.getByText(/deposit address:/i)).toHaveCount(0);
  expect(bodies.every((b) => b.dry === true)).toBe(true);
});
