# Frontend Deployment Guide (Vercel)

NeoNexus is a unified Next.js application containing both the Marketing Website and the Control Dashboard, optimized for seamless deployment on Vercel.

## 1. Database Setup (Neon)

1. Create an account on [Neon.tech](https://neon.tech).
2. Create a new project and select **Postgres 15+**.
3. Copy your `DATABASE_URL`. It should look like: `postgresql://user:password@db-host.example.com/neondb?sslmode=require`.
4. Locally, make sure you have pushed the schema to this database:
   ```bash
   cd dashboard
   DATABASE_URL="your-neon-url" npx prisma db push
   ```
5. Set `OPERATOR_EMAILS` in your environment to the comma-separated list of emails that should have platform operator access:
   ```env
   OPERATOR_EMAILS=you@example.com,ops@example.com
   ```
6. If you prefer persistent role assignment in the database, you can also promote your own account after the first login creates a user row:
   ```sql
   update "User"
   set role = 'operator'
   where email = 'you@example.com';
   ```
7. Re-run `npx prisma db push` whenever you pull schema changes from this repository. The dashboard now depends on the `User.role` column at runtime, even if you primarily bootstrap access with `OPERATOR_EMAILS`.

## 2. Authentication Setup (NextAuth)

### Generate a Secret
Generate a random 32-character string for NextAuth:
```bash
openssl rand -base64 32
```

### GitHub OAuth
1. Go to GitHub -> Settings -> Developer Settings -> OAuth Apps -> New OAuth App.
2. Homepage URL: `https://your-domain.com`
3. Authorization callback URL: `https://your-domain.com/api/auth/callback/github`
4. Save the **Client ID** and generate a **Client Secret**.

## 3. Stripe Setup (Billing)

1. Create a [Stripe](https://stripe.com) account.
2. Go to the Developer Dashboard and copy your **Secret Key** (`sk_live_...` or `sk_test_...`).
3. Create two Products (Subscriptions):
   - **Growth Plan** ($49/mo) -> Save the Price ID (e.g., `price_12345`)
   - **Dedicated Plan** ($99/mo) -> Save the Price ID (e.g., `price_67890`)
4. Set up a Webhook endpoint pointing to: `https://your-domain.com/api/webhooks/stripe`.
   - Listen for the `checkout.session.completed` event.
   - Save the **Webhook Signing Secret** (`whsec_...`).

## 4. Deploying to Vercel

1. Push your code to GitHub.
2. Log in to Vercel and click **Add New -> Project**.
3. Import your `r3e-network/neonexus` repository.
4. Leave the **Root Directory** empty (or set to `./`). Vercel will automatically detect the custom build scripts in the root `package.json`.
5. Add the following Environment Variables:
   - `DATABASE_URL`: (Your Neon DB URL)
   - `DIRECT_URL`: (Same as above, or non-pooled URL)
   - `NEXTAUTH_SECRET`: (From step 2)
   - `NEXTAUTH_URL`: `https://your-domain.com`
   - `GITHUB_ID`: (From step 2)
   - `GITHUB_SECRET`: (From step 2)
   - `OPERATOR_EMAILS`: comma-separated operator email allowlist for the platform-only Operations surfaces
   - `STRIPE_SECRET_KEY`: (From step 3)
   - `STRIPE_WEBHOOK_SECRET`: (From step 3)
   - `STRIPE_PRICE_ID_GROWTH`: (From step 3)
   - `STRIPE_PRICE_ID_DEDICATED`: (From step 3)
   - `APISIX_ADMIN_URL`
   - `APISIX_ADMIN_KEY`
   - `HETZNER_API_TOKEN`
   - `DIGITALOCEAN_API_TOKEN`
   - `SHARED_NEO_N3_MAINNET_UPSTREAM`
   - `SHARED_NEO_N3_TESTNET_UPSTREAM`
   - `SHARED_NEO_X_MAINNET_UPSTREAM`
   - `SHARED_NEO_X_TESTNET_UPSTREAM`
6. **(Important)** Dedicated node provisioning now depends on provider API credentials and APISIX. Leaving those blank will prevent real endpoint creation.
7. Click **Deploy**. Assign your custom domain (e.g., `neonexus.cloud`).

You now have a fully functional frontend architecture connected to a database and payment processor!
