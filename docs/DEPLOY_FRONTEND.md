# Frontend Deployment Guide (Vercel)

NeoNexus consists of two Next.js applications stored in a Monorepo setup. Both are optimized to be deployed seamlessly on Vercel.

## 1. Database Setup (Neon)

1. Create an account on [Neon.tech](https://neon.tech).
2. Create a new project and select **Postgres 15+**.
3. Copy your `DATABASE_URL`. It should look like: `postgresql://user:password@endpoint.aws.neon.tech/neondb?sslmode=require`.
4. Locally, make sure you have pushed the schema to this database:
   ```bash
   cd dashboard
   DATABASE_URL="your-neon-url" npx prisma db push
   ```

## 2. Authentication Setup (NextAuth)

### Generate a Secret
Generate a random 32-character string for NextAuth:
```bash
openssl rand -base64 32
```

### GitHub OAuth
1. Go to GitHub -> Settings -> Developer Settings -> OAuth Apps -> New OAuth App.
2. Homepage URL: `https://your-domain.com`
3. Authorization callback URL: `https://app.your-domain.com/api/auth/callback/github`
4. Save the **Client ID** and generate a **Client Secret**.

## 3. Stripe Setup (Billing)

1. Create a [Stripe](https://stripe.com) account.
2. Go to the Developer Dashboard and copy your **Secret Key** (`sk_live_...` or `sk_test_...`).
3. Create two Products (Subscriptions):
   - **Growth Plan** ($49/mo) -> Save the Price ID (e.g., `price_12345`)
   - **Dedicated Plan** ($99/mo) -> Save the Price ID (e.g., `price_67890`)
4. Set up a Webhook endpoint pointing to: `https://app.your-domain.com/api/webhooks/stripe`.
   - Listen for the `checkout.session.completed` event.
   - Save the **Webhook Signing Secret** (`whsec_...`).

## 4. Deploying to Vercel

1. Push your code to GitHub.
2. Log in to Vercel and click **Add New -> Project**.
3. Import your `r3e-network/neonexus` repository.

### Deploying the Website (Marketing)
1. In the Vercel project configuration, set the **Root Directory** to `website`.
2. Framework Preset will automatically be detected as `Next.js`.
3. Add the following Environment Variables:
   - `NEXTAUTH_SECRET`: (From step 2)
   - `NEXTAUTH_URL`: `https://your-domain.com`
   - `NEXT_PUBLIC_DASHBOARD_URL`: `https://app.your-domain.com`
4. Click **Deploy**. Assign your custom domain (e.g., `neonexus.io`).

### Deploying the Dashboard (Control Console)
1. Go back to the Vercel dashboard and click **Add New -> Project** again.
2. Import the same repository.
3. This time, set the **Root Directory** to `dashboard`.
4. Add the following Environment Variables:
   - `DATABASE_URL`: (Your Neon DB URL)
   - `DIRECT_URL`: (Same as above, or non-pooled URL)
   - `NEXTAUTH_SECRET`: (From step 2)
   - `NEXTAUTH_URL`: `https://app.your-domain.com`
   - `GITHUB_ID`: (From step 2)
   - `GITHUB_SECRET`: (From step 2)
   - `STRIPE_SECRET_KEY`: (From step 3)
   - `STRIPE_WEBHOOK_SECRET`: (From step 3)
   - `STRIPE_PRICE_ID_GROWTH`: (From step 3)
   - `STRIPE_PRICE_ID_DEDICATED`: (From step 3)
5. **(Important)** For Kubernetes integration, you must provide your cluster credentials. (See the Infrastructure Guide). For now, leaving these blank will put the dashboard into "Mock Deployment Mode".
   - `KUBECONFIG_BASE64` (Optional: If querying K8s from Vercel edge)
   - `APISIX_ADMIN_URL`
   - `APISIX_ADMIN_KEY`
6. Click **Deploy**. Assign a subdomain (e.g., `app.neonexus.io`).

You now have a fully functional frontend architecture connected to a database and payment processor!
