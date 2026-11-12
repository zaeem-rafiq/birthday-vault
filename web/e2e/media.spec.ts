import { expect, test } from "@playwright/test";

// Every vault NFT's metadata points at /vault.svg on the site (VAULT_MEDIA in
// contracts/collection/src/lib.rs). The collection has no keys, so the site must serve it.
test("vault NFT picture is served", async ({ request }) => {
  const res = await request.get("/vault.svg");
  expect(res.status()).toBe(200);
  expect(res.headers()["content-type"]).toContain("image/svg+xml");
  const body = await res.text();
  expect(body).toContain("<svg");
  expect(body.length).toBeLessThan(4096);
});
