# Infrastructure Deployment Guide (Control Plane)

This guide covers deploying the backend control-plane services that support NeoNexus.

NeoNexus now uses a VM-first dedicated-node architecture:
- **Hetzner** is the primary dedicated-node provider
- **DigitalOcean** is the fallback dedicated-node provider
- **APISIX** fronts both dedicated-node traffic and shared-node upstreams

## 1. Provision the control plane

1. Run APISIX and monitoring on a small control-plane environment. Kubernetes is one supported option, but it is only for shared services such as APISIX, Prometheus, and Grafana.
2. Prepare provider API credentials for:
   - `HETZNER_API_TOKEN`
   - `DIGITALOCEAN_API_TOKEN`
3. Prepare shared upstream targets for free/shared endpoints:
   - `SHARED_NEO_N3_MAINNET_UPSTREAM`
   - `SHARED_NEO_N3_TESTNET_UPSTREAM`
   - `SHARED_NEO_X_MAINNET_UPSTREAM`
   - `SHARED_NEO_X_TESTNET_UPSTREAM`

## 2. (Optional) Bootstrap a Kubernetes control-plane cluster

If you choose Kubernetes for APISIX and monitoring, use the provided bootstrap script. This script does not provision dedicated customer nodes. Dedicated nodes are still created directly as VMs through Hetzner or DigitalOcean.

1. Configure your local `kubectl` to point to the control-plane cluster.
2. Run the setup script from the root of this repository:

```bash
chmod +x scripts/setup_k8s.sh
CLUSTER_PROVIDER=hetzner ./scripts/setup_k8s.sh
```

Use `CLUSTER_PROVIDER=digitalocean` when the control-plane cluster runs on DigitalOcean. The provider flag only affects storage defaults for the control-plane services, not dedicated-node VM provisioning.

### What this script does:
- Creates `ingress-apisix` namespace and deploys the API gateway.
- Applies a provider-aware storage profile for the control-plane cluster.
- Creates `monitoring` namespace and deploys Grafana & Prometheus.

## 3. Configure APISIX Load Balancer

1. Retrieve the external IP of your APISIX ingress controller:
   ```bash
   kubectl get svc -n ingress-apisix apisix-gateway
   ```
2. Note the `EXTERNAL-IP`.
3. In your DNS provider, create records that cover both generated dedicated-node hosts and shared endpoint hosts:
   - `*.neonexus.cloud` -> `[EXTERNAL-IP]`
   - or an equivalent wildcard / delegated subdomain strategy that covers hostnames such as `node-fsn1-42.neonexus.cloud` and `mainnet.neonexus.cloud`

## 4. Connect the Dashboard to the control plane

1. Expose the APISIX admin endpoint securely.
2. Configure the dashboard environment with:
   - `APISIX_ADMIN_URL`
   - `APISIX_ADMIN_KEY`
   - `HETZNER_API_TOKEN`
   - `DIGITALOCEAN_API_TOKEN`
3. Configure the shared upstream targets used by the free/shared endpoint tier.
4. Configure `ENDPOINT_ROOT_DOMAIN` if you are not using `neonexus.cloud` for generated public endpoint hostnames.

## 5. (Optional) Helm Chart Validation

The Helm charts under `/infrastructure/helm` remain useful for validating shared-service or experimental cluster deployments. They are not the primary dedicated-node runtime path anymore.

```bash
# Create a test namespace
kubectl create namespace tenant-test-1

# Deploy a Neo N3 node (neo-go)
helm install neo-n3-test ./infrastructure/helm/neo-go \
  --namespace tenant-test-1 \
  --set network="mainnet" \
  --set persistence.size="200Gi"

# Or deploy a Neo X node (EVM)
helm install neo-x-test ./infrastructure/helm/neo-x \
  --namespace tenant-test-1 \
  --set network="mainnet" \
  --set persistence.size="500Gi"

# Watch the pod start and sync via initContainer snapshot
kubectl get pods -n tenant-test-1 -w
```
