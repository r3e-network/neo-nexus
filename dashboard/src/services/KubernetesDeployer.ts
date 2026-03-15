import * as k8s from '@kubernetes/client-node';
import * as yaml from 'js-yaml';

export interface DeploymentConfig {
    name: string;
    network: 'mainnet' | 'testnet';
    type: 'shared' | 'dedicated';
    clientEngine: 'neo-go' | 'neo-cli';
    provider: 'aws' | 'gcp';
    region: string;
    syncMode: 'full' | 'archive';
}

export class KubernetesDeployer {
    private static kc: k8s.KubeConfig;
    private static k8sApi: k8s.CoreV1Api;
    private static k8sAppsApi: k8s.AppsV1Api;
    private static initialized = false;

    private static init() {
        if (!this.initialized) {
            this.kc = new k8s.KubeConfig();
            try {
                // In production (inside a pod), this will load the service account token
                this.kc.loadFromCluster();
            } catch (e) {
                // Fallback to local kubeconfig for development
                try {
                    this.kc.loadFromDefault();
                } catch (err) {
                    console.warn('[KubernetesDeployer] Kubernetes config not found. Will run in mock/dry-run mode.');
                }
            }
            this.k8sApi = this.kc.makeApiClient(k8s.CoreV1Api);
            this.k8sAppsApi = this.kc.makeApiClient(k8s.AppsV1Api);
            this.initialized = true;
        }
    }

    /**
     * Translates a dashboard deployment request into Helm-equivalent K8s objects 
     * and deploys them directly to the cluster via API.
     */
    static async deployNode(config: DeploymentConfig): Promise<{ success: boolean; namespace: string; releaseName: string; error?: string }> {
        this.init();
        
        console.log(`[Control Plane] Initiating production deployment for ${config.name}...`);
        
        const uniqueId = Math.random().toString(36).substring(2, 8);
        const releaseName = `neo-${config.clientEngine}-${uniqueId}`;
        const namespace = `tenant-${uniqueId}`;

        // If no valid KubeConfig, return a successful mock for local dev
        if (!this.kc.getCurrentCluster()) {
            console.log('[Control Plane] Mock deployment successful (No K8s cluster configured).');
            return { success: true, namespace, releaseName };
        }

        try {
            // 1. Create Namespace
            const namespaceObj: k8s.V1Namespace = {
                metadata: { name: namespace, labels: { "neonexus/tenant": uniqueId } }
            };
            await this.k8sApi.createNamespace({ body: namespaceObj });

            // 2. Define StatefulSet Spec
            const isNeoGo = config.clientEngine === 'neo-go';
            const storageClass = config.provider === 'aws' ? 'gp3' : 'premium-rwo';
            const storageSize = config.syncMode === 'archive' ? '2Ti' : '200Gi';
            
            const statefulSetSpec: k8s.V1StatefulSet = {
                metadata: {
                    name: releaseName,
                    namespace: namespace,
                    labels: { app: releaseName }
                },
                spec: {
                    serviceName: releaseName,
                    replicas: 1,
                    selector: { matchLabels: { app: releaseName } },
                    template: {
                        metadata: { labels: { app: releaseName } },
                        spec: {
                            containers: [{
                                name: config.clientEngine,
                                image: isNeoGo ? 'nspccdev/neo-go:0.106.0' : 'neo-project/neo-cli:3.7.4',
                                command: isNeoGo 
                                    ? ["neo-go", "node", "--config-path", "/config", "--relative-path"]
                                    : ["dotnet", "neo-cli.dll", "--rpc"],
                                ports: [
                                    { containerPort: 10332, name: 'rpc' },
                                    { containerPort: 10333, name: 'p2p' },
                                    ...(isNeoGo ? [{ containerPort: 2112, name: 'metrics' }] : [])
                                ],
                                resources: {
                                    requests: {
                                        cpu: isNeoGo ? '2000m' : '4000m',
                                        memory: isNeoGo ? '4Gi' : '6Gi'
                                    },
                                    limits: {
                                        cpu: isNeoGo ? '4000m' : '8000m',
                                        memory: isNeoGo ? '8Gi' : '12Gi'
                                    }
                                },
                                volumeMounts: [
                                    { name: 'data', mountPath: isNeoGo ? '/data' : '/app/Data_LevelDB' }
                                ]
                            }]
                        }
                    },
                    volumeClaimTemplates: [{
                        metadata: { name: 'data' },
                        spec: {
                            accessModes: ['ReadWriteOnce'],
                            storageClassName: storageClass,
                            resources: { requests: { storage: storageSize } }
                        }
                    }]
                }
            };

            // 3. Define Service Spec
            const serviceSpec: k8s.V1Service = {
                metadata: {
                    name: releaseName,
                    namespace: namespace,
                },
                spec: {
                    selector: { app: releaseName },
                    ports: [
                        { port: 10332, targetPort: 10332, name: 'rpc' },
                        { port: 10333, targetPort: 10333, name: 'p2p' }
                    ],
                    type: 'ClusterIP'
                }
            };

            // 4. Execute Kubernetes API Calls
            await this.k8sAppsApi.createNamespacedStatefulSet({ namespace, body: statefulSetSpec });
            await this.k8sApi.createNamespacedService({ namespace, body: serviceSpec });

            console.log(`[Control Plane] Deployment ${releaseName} successfully scheduled in cluster.`);

            return { success: true, namespace, releaseName };

        } catch (error: any) {
            console.error(`[Control Plane] Deployment failed:`, error.response?.body || error.message);
            return { 
                success: false, 
                namespace, 
                releaseName, 
                error: error.response?.body?.message || error.message 
            };
        }
    }

    /**
     * Get the real-time status of the StatefulSet from Kubernetes
     */
    static async getNodeStatus(namespace: string, releaseName: string): Promise<'Syncing' | 'Active' | 'Error' | 'Unknown'> {
        this.init();
        if (!this.kc.getCurrentCluster()) return 'Active'; // Mock fallback

        try {
            const res = await this.k8sAppsApi.readNamespacedStatefulSet({ name: releaseName, namespace });
            const status = res.status;
            
            if (!status) return 'Unknown';
            if (status.readyReplicas === status.replicas) {
                return 'Active';
            }
            if (status.replicas && (!status.readyReplicas || status.readyReplicas < status.replicas)) {
                return 'Syncing';
            }
            return 'Error';
        } catch (error) {
            console.error(`[Control Plane] Error fetching status for ${releaseName}:`, error);
            return 'Error';
        }
    }
}
