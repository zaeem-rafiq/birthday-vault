# Owner steps

These are the steps only the owner takes. Claude prepares them and never handles a Render API key or a wallet secret.

## Put the public site live on Render (after you reply PUBLISH)

You need: a Render account signed in to GitHub, and the file `.env.render.local` in the project folder on the Mac. Claude tells you when the final testnet collection id is ready.

1. Open https://dashboard.render.com and sign in.
2. Click **New**, then **Static Site**.
3. Pick the repository `zaeem-rafiq/birthday-vault`. If it is not listed, click **Configure account** and give Render access to it.
4. In **Name**, type `birthday-vault`.
5. In **Branch**, choose `main`.
6. In **Root Directory**, type `web`.
7. In **Build Command**, type `npm ci && npm run build`.
8. In **Publish Directory**, type `out`.
9. Open **Advanced**, then add these environment variables. Copy each value from `.env.render.local`, except the last three:
   - `NEXT_PUBLIC_NETWORK`
   - `NEXT_PUBLIC_HUB_ID`
   - `NEXT_PUBLIC_USDC_ID`
   - `NEXT_PUBLIC_HUB_KEYS`
   - `NEXT_PUBLIC_COLLECTION_ID`: the collection id Claude gives you.
   - `NODE_VERSION`: `26.11.0`
   - `NEXT_TELEMETRY_DISABLED`: `1`
10. Click **Deploy Static Site**.
11. Wait until the page shows **Live** with a green dot. This takes about 3 minutes.
12. Copy the address that ends in `.onrender.com` and send it to Claude.

The hub keys in `NEXT_PUBLIC_HUB_KEYS` are public on purpose. They can only start testnet demos, and the hub's daily limit caps their cost to testnet NEAR.
