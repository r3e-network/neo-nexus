#!/bin/bash
# NeoNexus Optional Kubernetes Control-Plane Bootstrap Script
# This script sets up APISIX, monitoring, and provider-aware storage defaults
# on a Kubernetes cluster used only for shared control-plane services.

set -e

CLUSTER_PROVIDER="${CLUSTER_PROVIDER:-hetzner}"

if [[ "$CLUSTER_PROVIDER" != "hetzner" && "$CLUSTER_PROVIDER" != "digitalocean" ]]; then
  echo "Unsupported CLUSTER_PROVIDER: $CLUSTER_PROVIDER"
  echo "Use one of: hetzner, digitalocean"
  exit 1
fi

echo "🚀 Initializing optional NeoNexus Kubernetes control-plane services..."
echo "Provider profile: $CLUSTER_PROVIDER"

# 1. Install Helm
if ! command -v helm &> /dev/null
then
    echo "Installing Helm..."
    curl https://raw.githubusercontent.com/helm/helm/main/scripts/get-helm-3 | bash
fi

# 2. Install APISIX Ingress Controller
echo "📦 Installing Apache APISIX Gateway..."
helm repo add apisix https://charts.apiseven.com
helm repo update

# Install APISIX in its own namespace
helm upgrade --install apisix apisix/apisix \
  --namespace ingress-apisix \
  --create-namespace \
  --set gateway.type=LoadBalancer \
  --set admin.credentials.admin=***REMOVED*** \
  --set ingress-controller.enabled=true \
  --set ingress-controller.config.apisix.adminKey="***REMOVED***"

# 3. Setup Storage Classes
echo "💾 Configuring storage profile..."
if [[ "$CLUSTER_PROVIDER" == "hetzner" ]]; then
kubectl apply -f - <<EOF
apiVersion: storage.k8s.io/v1
kind: StorageClass
metadata:
  name: hcloud-volumes
provisioner: csi.hetzner.cloud
volumeBindingMode: WaitForFirstConsumer
allowVolumeExpansion: true
reclaimPolicy: Delete
EOF
  echo "Applied hcloud-volumes StorageClass for Hetzner."
else
  echo "DigitalOcean profile selected. Expect built-in do-block-storage to exist already."
  kubectl get storageclass do-block-storage >/dev/null
fi

# 4. Setup Prometheus & Grafana Operator for Observability
echo "📊 Installing kube-prometheus-stack..."
helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
helm repo update
helm upgrade --install monitoring prometheus-community/kube-prometheus-stack \
  --namespace monitoring \
  --create-namespace \
  --set grafana.adminPassword=neonexus_admin

echo "✅ Optional NeoNexus Kubernetes control-plane services initialized successfully."
echo "APISIX Admin URL is exposed internally. Next steps: configure dashboard control-plane environment variables."
