-- NeoNexus Supabase PostgreSQL Schema

-- Users / Organizations (Extends Supabase auth.users)
create table public.organizations (
    id uuid references auth.users not null primary key,
    name text not null,
    billing_plan text default 'developer' check (billing_plan in ('developer', 'growth', 'dedicated', 'enterprise')),
    stripe_customer_id text,
    created_at timestamp with time zone default timezone('utc'::text, now()) not null
);

-- Endpoints (Nodes)
create table public.endpoints (
    id bigint generated always as identity primary key,
    user_id uuid references public.organizations(id),
    name text not null,
    network text not null check (network in ('N3 Mainnet', 'N3 Testnet', 'Neo X Mainnet', 'Neo X Testnet')),
    type text not null check (type in ('Shared', 'Dedicated')),
    client_engine text default 'neo-go' check (client_engine in ('neo-go', 'neo-cli', 'neo-x-geth')),
    url text not null,
    wss_url text,
    status text not null default 'Syncing' check (status in ('Syncing', 'Active', 'Stopped', 'Error')),
    requests bigint default 0,
    k8s_namespace text,
    k8s_deployment_name text,
    cloud_provider text check (cloud_provider in ('aws', 'gcp')),
    region text,
    created_at timestamp with time zone default timezone('utc'::text, now()) not null
);

-- API Keys
create table public.api_keys (
    id uuid default gen_random_uuid() primary key,
    user_id uuid references public.organizations(id),
    name text not null,
    key_hash text not null, -- Store hash, not raw key
    is_active boolean default true,
    created_at timestamp with time zone default timezone('utc'::text, now()) not null
);

-- Firewalls (IP Allowlist & Method Rules)
create table public.firewalls (
    id bigint generated always as identity primary key,
    endpoint_id bigint references public.endpoints(id) on delete cascade,
    type text not null check (type in ('ip_allow', 'method_block', 'origin_allow')),
    value text not null,
    created_at timestamp with time zone default timezone('utc'::text, now()) not null
);

-- Row Level Security (RLS)
alter table public.organizations enable row level security;
alter table public.endpoints enable row level security;
alter table public.api_keys enable row level security;
alter table public.firewalls enable row level security;

-- Policies (Users can only see their own data)
create policy "Users can view own organization" on public.organizations for select using (auth.uid() = id);
create policy "Users can view own endpoints" on public.endpoints for select using (auth.uid() = user_id);
create policy "Users can insert own endpoints" on public.endpoints for insert with check (auth.uid() = user_id);
create policy "Users can view own api_keys" on public.api_keys for select using (auth.uid() = user_id);
create policy "Users can view own firewalls" on public.firewalls for select using (
    exists (select 1 from public.endpoints e where e.id = firewalls.endpoint_id and e.user_id = auth.uid())
);
