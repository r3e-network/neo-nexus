import * as k8s from '@kubernetes/client-node';
import { getErrorMessage, getKubernetesErrorMessage } from '@/server/errors';

export interface DeploymentConfig {
    name: string;
    protocol: 'neo-n3' | 'neo-x';
    network: 'mainnet' | 'testnet' | 'private';
    type: 'shared' | 'dedicated';
    clientEngine: 'neo-go' | 'neo-cli' | 'neo-x-geth';
    provider: 'aws' | 'gcp' | 'digitalocean';
    region: string;
    syncMode: 'full' | 'archive';
}

export class KubernetesDeployer {
    private static kc: k8s.KubeConfig;
    private static k8sApi: k8s.CoreV1Api;
    private static k8sAppsApi: k8s.AppsV1Api;
    private static initialized = false;
    private static configured = false;

    private static init() {
        if (!this.initialized) {
            this.kc = new k8s.KubeConfig();
            try {
                // In production (inside a pod), this will load the service account token
                this.kc.loadFromCluster();
                this.configured = true;
            } catch {
                // Fallback to local kubeconfig for development
                try {
                    this.kc.loadFromDefault();
                    this.configured = true;
                } catch {
                    console.warn('[KubernetesDeployer] Kubernetes config not found. Will run in dev/dry-run mode.');
                }
            }
            if (this.configured) {
                this.k8sApi = this.kc.makeApiClient(k8s.CoreV1Api);
                this.k8sAppsApi = this.kc.makeApiClient(k8s.AppsV1Api);
            }
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
        const releaseName = `${config.protocol}-${config.clientEngine}-${uniqueId}`;
        const namespace = `tenant-${uniqueId}`;

        // In a true production environment, we enforce Kubernetes configuration.
        if (!this.configured || !this.kc.getCurrentCluster()) {
            throw new Error('[Control Plane] Kubernetes cluster configuration is missing or invalid.');
        }

        try {
            // 1. Create Namespace
            const namespaceObj: k8s.V1Namespace = {
                metadata: { name: namespace, labels: { "neonexus/tenant": uniqueId } }
            };
            await this.k8sApi.createNamespace({ body: namespaceObj });

            // 2. Define StatefulSet Spec
            const isNeoGo = config.clientEngine === 'neo-go';
            const isNeoX = config.protocol === 'neo-x';
            
            let storageClass = 'do-block-storage'; // Default to DigitalOcean Block Storage
            if (config.provider === 'aws') storageClass = 'gp3';
            if (config.provider === 'gcp') storageClass = 'premium-rwo';

            const storageSize = config.syncMode === 'archive' ? '2Ti' : (isNeoX ? '500Gi' : '200Gi');
            
            // Example official snapshot URL - in a real production environment, this should point to a regional S3 bucket
            const snapshotUrl = isNeoX 
                ? 'https://sync.neox.network/mainnet/geth-chain.tar.gz' 
                : 'https://sync.ngd.network/mainnet/chain.acc.tar.gz';
                
            const dataPath = isNeoX ? '/data/geth' : (isNeoGo ? '/data' : '/app/Data_LevelDB');

            let containerImage = '';
            let containerCommand: string[] = [];
            let containerPorts: k8s.V1ContainerPort[] = [];
            let cpuReq = '2000m', memReq = '4Gi', cpuLim = '4000m', memLim = '8Gi';

            if (isNeoX) {
                containerImage = 'neofoundation/neo-x-geth:latest';
                containerCommand = ["geth", "--http", "--http.addr", "0.0.0.0", "--http.port", "8545", "--ws", "--ws.addr", "0.0.0.0", "--ws.port", "8546", "--datadir", dataPath];
                containerPorts = [
                    { containerPort: 8545, name: 'rpc' },
                    { containerPort: 8546, name: 'ws' },
                    { containerPort: 30303, name: 'p2p' }
                ];
                cpuReq = '4000m'; memReq = '8Gi'; cpuLim = '8000m'; memLim = '16Gi'; // EVM needs more juice
            } else if (isNeoGo) {
                containerImage = 'nspccdev/neo-go:0.106.0';
                containerCommand = ["neo-go", "node", "--config-path", "/config", "--relative-path"];
                containerPorts = [
                    { containerPort: 10332, name: 'rpc' },
                    { containerPort: 10333, name: 'p2p' },
                    { containerPort: 2112, name: 'metrics' }
                ];
            } else {
                // neo-cli
                containerImage = 'neo-project/neo-cli:3.7.4';
                containerCommand = ["dotnet", "neo-cli.dll", "--rpc"];
                containerPorts = [
                    { containerPort: 10332, name: 'rpc' },
                    { containerPort: 10333, name: 'p2p' }
                ];
                cpuReq = '4000m'; memReq = '6Gi'; cpuLim = '8000m'; memLim = '12Gi';
            }

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
                            initContainers: [
                                {
                                    name: 'init-snapshot',
                                    image: 'alpine:latest',
                                    command: ['/bin/sh', '-c'],
                                    args: [
                                        `if [ ! -d "${dataPath}/chains" ] && [ ! -d "${dataPath}/Data_LevelDB" ] && [ ! -d "${dataPath}/chaindata" ]; then
                                            echo "Downloading snapshot from ${snapshotUrl}..."
                                            wget -O /tmp/snapshot.tar.gz ${snapshotUrl}
                                            echo "Extracting snapshot..."
                                            tar -xzf /tmp/snapshot.tar.gz -C ${dataPath}
                                            rm /tmp/snapshot.tar.gz
                                            echo "Snapshot initialized."
                                        else
                                            echo "Data directory already initialized, skipping snapshot."
                                        fi`
                                    ],
                                    volumeMounts: [{ name: 'data', mountPath: isNeoX ? '/data' : dataPath }]
                                }
                            ],
                            containers: [{
                                name: config.clientEngine,
                                image: containerImage,
                                command: containerCommand,
                                ports: containerPorts,
                                resources: {
                                    requests: { cpu: cpuReq, memory: memReq },
                                    limits: { cpu: cpuLim, memory: memLim }
                                },
                                volumeMounts: [
                                    { name: 'data', mountPath: isNeoX ? '/data' : dataPath }
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
                    ports: isNeoX ? [
                        { port: 8545, targetPort: 8545, name: 'rpc' },
                        { port: 8546, targetPort: 8546, name: 'ws' },
                        { port: 30303, targetPort: 30303, name: 'p2p' }
                    ] : [
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

        } catch (error) {
            console.error('[Control Plane] Deployment failed:', error);
            return { 
                success: false, 
                namespace, 
                releaseName, 
                error: getKubernetesErrorMessage(error) 
            };
        }
    }

    /**
     * Get the real-time status of the StatefulSet from Kubernetes
     */
    static async getNodeStatus(namespace: string, releaseName: string): Promise<'Syncing' | 'Active' | 'Error' | 'Unknown'> {
        this.init();
        if (!this.configured || !this.kc.getCurrentCluster()) {
            console.error('[Control Plane] Kubernetes cluster configuration is missing or invalid.');
            return 'Unknown';
        }

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
            console.error(`[Control Plane] Error fetching status for ${releaseName}:`, getErrorMessage(error));
            return 'Error';
        }
    }
}
