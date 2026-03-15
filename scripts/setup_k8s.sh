#!/bin/bash
# NeoNexus Production Kubernetes Cluster Initialization Script
# This script sets up the required dependencies on a blank EKS or GKE cluster.

set -e

echo "🚀 Initializing NeoNexus Kubernetes Cluster Configuration..."

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
  --set admin.credentials.admin=edd1c9f034335f136f87ad84b625c8f1 \
  --set ingress-controller.enabled=true \
  --set ingress-controller.config.apisix.adminKey="edd1c9f034335f136f87ad84b625c8f1"

# 3. Setup Storage Classes
echo "💾 Setting up high-performance storage classes..."
kubectl apply -f - <<EOF
apiVersion: storage.k8s.io/v1
kind: StorageClass
metadata:
  name: gp3
provisioner: ebs.csi.aws.com
volumeBindingMode: WaitForFirstConsumer
parameters:
  type: gp3
  iops: "3000"
  throughput: "125"
EOF

# 4. Setup Prometheus & Grafana Operator for Observability
echo "📊 Installing kube-prometheus-stack..."
helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
helm repo update
helm upgrade --install monitoring prometheus-community/kube-prometheus-stack \
  --namespace monitoring \
  --create-namespace \
  --set grafana.adminPassword=neonexus_admin

echo "✅ NeoNexus Cluster Base Infrastructure Initialized successfully!"
echo "APISIX Admin URL is exposed internally. Next steps: Configure Control Plane Environment variables."
