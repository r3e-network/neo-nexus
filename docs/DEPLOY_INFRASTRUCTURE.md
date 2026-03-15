# Infrastructure Deployment Guide (Kubernetes)

This guide covers deploying the backend "Control Plane" of NeoNexus. This is where the actual Neo N3 and Neo X blockchain nodes run.

## 1. Provision a Kubernetes Cluster

You must use a managed Kubernetes service that supports fast block storage provisioning.

### Recommended: AWS EKS
1. Create an EKS cluster (version 1.28+).
2. Create a managed Node Group.
   - Instance Type: `m6i.xlarge` or `m6a.xlarge` (At least 4 vCPUs, 16GB RAM). Neo X (EVM) nodes are especially resource-intensive.
   - *Blockchain nodes are CPU and I/O intensive.*
3. Ensure the AWS EBS CSI Driver add-on is installed on your cluster (required for `gp3` persistent volumes).

## 2. Initialize the Cluster

We have provided a bootstrap script that installs Helm, Apache APISIX (API Gateway), and the Prometheus Observability stack.

1. Configure your local `kubectl` to point to your new EKS cluster.
2. Run the setup script from the root of this repository:

```bash
chmod +x infrastructure/scripts/setup_k8s.sh
./infrastructure/scripts/setup_k8s.sh
```

### What this script does:
- Creates `ingress-apisix` namespace and deploys the API gateway.
- Creates a high-performance `gp3` StorageClass.
- Creates `monitoring` namespace and deploys Grafana & Prometheus.

## 3. Configure APISIX Load Balancer

1. Retrieve the external IP of your APISIX ingress controller:
   ```bash
   kubectl get svc -n ingress-apisix apisix-gateway
   ```
2. Note the `EXTERNAL-IP` (an AWS ELB address).
3. In your DNS provider (e.g., Route53, Cloudflare), create a wildcard CNAME record pointing to this Load Balancer:
   - `*.node.neonexus.io` -> `[EXTERNAL-IP]`
   - `mainnet.neonexus.io` -> `[EXTERNAL-IP]`

## 4. Connect the Dashboard to the Cluster

For the Vercel-hosted Dashboard to deploy nodes to your private EKS cluster, it needs authentication.

Because Vercel functions run statelessly on Edge/Serverless environments, the `@kubernetes/client-node` library needs the raw Kubeconfig.

1. Base64 encode your `~/.kube/config` file:
   ```bash
   cat ~/.kube/config | base64 | tr -d '\n'
   ```
2. Go to your Vercel Dashboard project settings -> Environment Variables.
3. Add a new variable:
   - Key: `KUBECONFIG_BASE64`
   - Value: *(The base64 string from step 1)*
4. Retrieve the internal APISIX Admin IP (or expose it securely) and add:
   - Key: `APISIX_ADMIN_URL`
   - Value: `http://[APISIX-ADMIN-IP]:9180/apisix/admin`
   - Key: `APISIX_ADMIN_KEY`
   - Value: `edd1c9f034335f136f87ad84b625c8f1` (Default from script, change in production).

## 5. (Optional) Manual Helm Deployment Test

You can test the deployment mechanism manually by using the provided Helm charts for Neo N3 (`neo-go` / `neo-cli`) or Neo X (`neo-x`).

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
