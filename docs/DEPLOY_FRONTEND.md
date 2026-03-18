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
5. Set `OPERATOR_WALLETS` in your environment to the comma-separated list of Neo N3 addresses that should have platform operator access:
   ```env
   OPERATOR_WALLETS=NNR...123,Nabc...xyz
   ```
6. If you prefer persistent role assignment in the database, you can also promote your own account after the first login creates a user row:
   ```sql
   update "User"
   set role = 'operator'
   where "walletAddress" = 'NNR...123';
   ```

## 2. Authentication Setup (NextAuth & Web3)

### Generate a Secret
Generate a random 32-character string for NextAuth session cookies:
```bash
openssl rand -base64 32
```
This becomes your `AUTH_SECRET` environment variable.

The platform uses **Sign-In with Neo (SIWN)**. No OAuth apps (GitHub/Google) are required! Users simply connect their NeoLine wallet to authenticate.

## 3. Cryptography & Custody (AWS KMS)

To securely encrypt plugin secrets (like Consensus and Oracle keys) before storing them in the database:
1. Create a Symmetric Encryption Key in **AWS KMS**.
2. Save the ARN of the key as `KMS_KEY_ID`.
3. Provide `AWS_ACCESS_KEY_ID` and `AWS_SECRET_ACCESS_KEY` to the environment if deploying outside of an IAM-role-assumed container.

*(Fallback: If you do not have AWS KMS, you can provide a 32-byte hex string as `VAULT_ENCRYPTION_KEY` for local symmetric encryption).*

## 4. Deploying to Vercel

1. Push your code to GitHub.
2. Log in to Vercel and click **Add New -> Project**.
3. Import your `r3e-network/neonexus` repository.
4. Leave the **Root Directory** empty (or set to `./`). Vercel will automatically detect the custom build scripts in the root `package.json`.
5. Add the following Environment Variables:
   - `DATABASE_URL`: (Your Neon DB URL)
   - `DIRECT_URL`: (Same as above, or non-pooled URL)
   - `AUTH_SECRET`: (From step 2)
   - `OPERATOR_WALLETS`: (From step 1)
   - `KMS_KEY_ID`: (From step 3)
   - `APISIX_ADMIN_URL`
   - `APISIX_ADMIN_KEY`
   - `NEO_NEXUS_HETZNER` (Your Hetzner API Token)
   - `DIGITALOCEAN_API_TOKEN`
   - `VM_OPERATOR_PUBLIC_KEY` (e.g. `ssh-ed25519 AAAAC3...`)
   - `SHARED_NEO_N3_MAINNET_UPSTREAM`
   - `SHARED_NEO_N3_TESTNET_UPSTREAM`
   - `SHARED_NEO_X_MAINNET_UPSTREAM`
   - `SHARED_NEO_X_TESTNET_UPSTREAM`
6. **(Important)** Dedicated node provisioning depends on provider API credentials, SSH keys, and APISIX. Leaving those blank will prevent real endpoint creation.
7. Click **Deploy**. Assign your custom domain (e.g., `neonexus.cloud`).

You now have a fully functional Node-as-a-Service architecture!
