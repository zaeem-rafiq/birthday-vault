# Owner steps

These are the steps only the owner takes. Claude prepares them and never handles a Render API key or a wallet secret.

## Put the public site live on Render (after you reply PUBLISH)

You need: a Render account signed in to GitHub, and the file `.env.render.local` in the project folder on the Mac.

The public contract ids are already in the code (`web/.env.production`): the collection `g2.birthday-vault-zr1109.testnet`, the hub and the play money. Render needs only the values below.

1. Open https://dashboard.render.com and sign in.
2. Click **New**, then **Static Site**.
3. Pick the repository `zaeem-rafiq/birthday-vault`. If it is not listed, click **Configure account** and give Render access to it.
4. In **Name**, type `birthday-vault`.
5. In **Branch**, choose `main`.
6. In **Root Directory**, type `web`.
7. In **Build Command**, type `npm ci && npm run build`.
8. In **Publish Directory**, type `out`.
9. Open **Advanced**, then add these three environment variables:
   - `NEXT_PUBLIC_HUB_KEYS`: copy the value from `.env.render.local`.
   - `NODE_VERSION`: `26.11.0`
   - `NEXT_TELEMETRY_DISABLED`: `1`
10. Click **Deploy Static Site**.
11. Wait until the page shows **Live** with a green dot. This takes about 3 minutes.
12. Check the address at the top of the page. It must be exactly `https://birthday-vault.onrender.com`. If Render shows any other address, stop and tell Claude: every vault NFT's picture points at that exact address, and it cannot change.
13. Send the address to Claude.

The hub keys in `NEXT_PUBLIC_HUB_KEYS` are public on purpose. They can only start testnet demos, and the hub's daily limit caps their cost to testnet NEAR.
