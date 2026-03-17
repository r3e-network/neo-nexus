'use client';

import { useRouter } from 'next/navigation';
import { useState, useEffect } from 'react';
import Link from 'next/link';
import { ArrowLeft, Activity, Box, Copy, Globe, MoreVertical, Power, RotateCcw, Server, Terminal, Lock, Plug, Mail, Webhook, Cpu, HardDrive, RefreshCw, Download } from 'lucide-react';
import { Endpoint } from '../EndpointsList';
import { NeoNodeService } from '@/services/neo/NeoNodeService';
import { addNodePluginAction, removeNodePluginAction } from '../pluginActions';
import { createAlertRuleAction, deleteAlertRuleAction } from '../alertActions';
import {
  destroyEndpointAction,
  restartEndpointAction,
  stopEndpointAction,
} from '../lifecycleActions';
import {
  exportEndpointSnapshotAction,
  triggerEndpointResyncAction,
  updateEndpointSettingsAction,
} from '../settingsActions';
import toast from 'react-hot-toast';
import { listAlertRuleDefinitions } from '@/services/alerts/AlertRuleCatalog';
import { deriveEndpointConnectionDisplay } from '@/services/endpoints/EndpointConnectionDisplay';
import { formatEndpointActivityMetadata } from '@/services/endpoints/EndpointActivityFormatter';
import { getSupportedNodeSettingKeys } from '@/services/settings/RenderedNodeRuntime';
import {
  listSupportedPlugins,
} from '@/services/plugins/PluginCatalog';

type ProvisioningOrderState = {
  status: string;
  currentStep: string;
  provider: string;
  attemptCount?: number;
  lastAttemptAt?: string | null;
  nextAttemptAt?: string | null;
  errorMessage?: string | null;
};

type NodePluginView = {
  id: number;
  pluginId: string;
  status: string;
  errorMessage: string | null;
  lastAppliedAt: string | null;
  createdAt: string;
};

type AlertRuleView = {
  id: number;
  name: string;
  condition: string;
  actionType: 'email' | 'webhook';
  target: string;
  isActive: boolean;
  createdAt: string;
  deliveryAttemptCount: number;
  nextDeliveryAt: string | null;
  lastTriggeredAt: string | null;
  lastDeliveredAt: string | null;
  lastResolvedAt: string | null;
  lastDeliveryError: string | null;
  triggered: boolean;
  severity: string;
  message: string;
};

type AlertIncidentView = {
  id: number;
  alertRuleId: number;
  status: string;
  severity: string;
  message: string;
  openedAt: string;
  resolvedAt: string | null;
  lastDeliveredAt: string | null;
  lastDeliveryError: string | null;
};

type NodeMetricsView = {
  cpuPercent: number | null;
  memoryUsedBytes: number | null;
  memoryLimitBytes: number | null;
  networkRxBytes: number | null;
  networkTxBytes: number | null;
  blockReadBytes: number | null;
  blockWriteBytes: number | null;
  diskUsedBytes: number | null;
  diskTotalBytes: number | null;
  updatedAt: string;
};

type NodeSettingsView = {
  maxPeers: number;
  rpcEnabled: boolean;
  websocketEnabled: boolean;
  graphqlEnabled: boolean;
  cacheMb: number | null;
};

type EndpointActivityView = {
  id: number;
  endpointId: number;
  category: string;
  action: string;
  status: string;
  message: string;
  metadata: unknown;
  createdAt: string;
};

type PendingConfirmation =
  | {
      kind: 'remove-plugin';
      pluginId: string;
      title: string;
      description: string;
      confirmLabel: string;
    }
  | {
      kind: 'resync';
      title: string;
      description: string;
      confirmLabel: string;
    }
  | {
      kind: 'destroy';
      title: string;
      description: string;
      confirmLabel: string;
    };

export default function EndpointDetailsClient({
  endpoint,
  provisioningOrderId,
  nodePlugins,
  nodeSettings,
}: {
  endpoint: Endpoint | null;
  provisioningOrderId?: number | null;
  nodePlugins: NodePluginView[];
  nodeSettings: NodeSettingsView;
}) {
  const router = useRouter();
  const [activeTab, setActiveTab] = useState('overview');
  const [blockHeight, setBlockHeight] = useState<number | string>('Syncing...');
  const [peerCount, setPeerCount] = useState<number | string>('Syncing...');
  const [installPluginModal, setInstallPluginModal] = useState<string | null>(null);
  const [privateKey, setPrivateKey] = useState('');
  const [pluginConfigInput, setPluginConfigInput] = useState('');
  const [isInstalling, setIsInstalling] = useState(false);
  const [isRemovingPlugin, setIsRemovingPlugin] = useState<string | null>(null);
  const [pluginLogsModal, setPluginLogsModal] = useState<string | null>(null);
  const [pluginLogs, setPluginLogs] = useState('');
  const [isLoadingPluginLogs, setIsLoadingPluginLogs] = useState(false);
  const [nodeLogs, setNodeLogs] = useState('Loading node logs...');
  const [isLoadingNodeLogs, setIsLoadingNodeLogs] = useState(false);
  const [nodeMetrics, setNodeMetrics] = useState<NodeMetricsView | null>(null);
  const [nodeMetricsError, setNodeMetricsError] = useState<string | null>(null);
  const [activities, setActivities] = useState<EndpointActivityView[]>([]);
  const [alerts, setAlerts] = useState<AlertRuleView[]>([]);
  const [alertIncidents, setAlertIncidents] = useState<AlertIncidentView[]>([]);
  const [isCreatingAlert, setIsCreatingAlert] = useState(false);
  const [newAlertCondition, setNewAlertCondition] = useState('endpoint_down');
  const [newAlertActionType, setNewAlertActionType] = useState<'email' | 'webhook'>('email');
  const [newAlertTarget, setNewAlertTarget] = useState('');
  const [deletingAlertId, setDeletingAlertId] = useState<number | null>(null);
  const [settingsForm, setSettingsForm] = useState<NodeSettingsView>(nodeSettings);
  const [isSavingSettings, setIsSavingSettings] = useState(false);
  const [maintenanceAction, setMaintenanceAction] = useState<string | null>(null);
  const [orderState, setOrderState] = useState<ProvisioningOrderState | null>(null);
  const [isLifecycleLoading, setIsLifecycleLoading] = useState<string | null>(null);
  const [pluginState, setPluginState] = useState<NodePluginView[]>(nodePlugins);
  const [pendingConfirmation, setPendingConfirmation] = useState<PendingConfirmation | null>(null);

  useEffect(() => {
    const fetchNodeStats = async () => {
      if (endpoint?.url && endpoint.status.toLowerCase() === 'active') {
        const height = await NeoNodeService.getBlockCount(endpoint.url, endpoint.clientEngine);
        if (height !== null) setBlockHeight(height.toLocaleString());

        const peers = await NeoNodeService.getPeersCount(endpoint.url, endpoint.clientEngine);
        if (peers !== null) setPeerCount(peers);
      } else if (endpoint?.status.toLowerCase() !== 'active') {
        setBlockHeight(endpoint?.status || 'Unknown');
        setPeerCount('-');
      }
    };

    fetchNodeStats();
    
    // Poll every 15 seconds if active
    const interval = endpoint?.status.toLowerCase() === 'active'
      ? setInterval(fetchNodeStats, 15000)
      : undefined;

    return () => clearInterval(interval);
  }, [endpoint]);

  useEffect(() => {
    if (!provisioningOrderId || endpoint?.status.toLowerCase() === 'active') {
      return;
    }

    const fetchOrder = async () => {
      const response = await fetch(`/api/orders/${provisioningOrderId}`);
      if (!response.ok) {
        return;
      }

      const data = await response.json() as ProvisioningOrderState;
      setOrderState(data);
    };

    fetchOrder();
    const interval = setInterval(fetchOrder, 5000);

    return () => clearInterval(interval);
  }, [endpoint?.status, provisioningOrderId]);

  useEffect(() => {
    if (!endpoint?.id) {
      return;
    }

    const fetchPlugins = async () => {
      const response = await fetch(`/api/endpoints/${endpoint.id}/plugins`);
      if (!response.ok) {
        return;
      }

      const data = await response.json() as NodePluginView[];
      setPluginState(data);
    };

    fetchPlugins();
    const interval = setInterval(fetchPlugins, 5000);

    return () => clearInterval(interval);
  }, [endpoint?.id]);

  useEffect(() => {
    if (activeTab !== 'metrics' || !endpoint?.id) {
      return;
    }

    const fetchMetrics = async () => {
      const response = await fetch(`/api/endpoints/${endpoint.id}/metrics`);
      const data = await response.json() as NodeMetricsView & { error?: string };

      if (!response.ok) {
        setNodeMetrics(null);
        setNodeMetricsError(data.error || 'Failed to fetch node metrics.');
        return;
      }

      setNodeMetricsError(null);
      setNodeMetrics(data);
    };

    fetchMetrics();
    const interval = setInterval(fetchMetrics, 5000);

    return () => clearInterval(interval);
  }, [activeTab, endpoint?.id]);

  useEffect(() => {
    if (activeTab !== 'alerts' || !endpoint?.id) {
      return;
    }

    const fetchAlerts = async () => {
      const response = await fetch(`/api/endpoints/${endpoint.id}/alerts`);
      if (!response.ok) {
        return;
      }

      const data = await response.json() as {
        rules: AlertRuleView[];
        incidents: AlertIncidentView[];
      };
      setAlerts(data.rules);
      setAlertIncidents(data.incidents);
    };

    fetchAlerts();
    const interval = setInterval(fetchAlerts, 5000);

    return () => clearInterval(interval);
  }, [activeTab, endpoint?.id]);

  useEffect(() => {
    if (activeTab !== 'activity' || !endpoint?.id) {
      return;
    }

    const fetchActivity = async () => {
      const response = await fetch(`/api/endpoints/${endpoint.id}/activity`);
      if (!response.ok) {
        return;
      }

      const data = await response.json() as EndpointActivityView[];
      setActivities(data);
    };

    fetchActivity();
    const interval = setInterval(fetchActivity, 5000);

    return () => clearInterval(interval);
  }, [activeTab, endpoint?.id]);

  useEffect(() => {
    if (activeTab !== 'logs' || !endpoint?.id) {
      return;
    }

    const fetchLogs = async () => {
      setIsLoadingNodeLogs(true);
      const response = await fetch(`/api/endpoints/${endpoint.id}/logs`);
      const data = await response.json() as { logs?: string; error?: string };
      if (!response.ok) {
        setNodeLogs(data.error || 'Failed to fetch node logs.');
        setIsLoadingNodeLogs(false);
        return;
      }

      setNodeLogs(data.logs || 'No logs returned.');
      setIsLoadingNodeLogs(false);
    };

    fetchLogs();
    const interval = setInterval(fetchLogs, 5000);

    return () => clearInterval(interval);
  }, [activeTab, endpoint?.id]);

  const handleInstallPlugin = async () => {
    if (!installPluginModal) return;
    if (!privateKey) {
        toast.error('A private key is required to configure this plugin.');
        return;
    }

    setIsInstalling(true);
    toast.loading(`Deploying ${installPluginModal} sidecar...`, { id: 'plugin' });

    const configPayload = pluginConfigInput.trim()
      ? { endpointUrl: endpoint?.url, note: pluginConfigInput.trim() }
      : { endpointUrl: endpoint?.url };

    const result = await addNodePluginAction(endpoint?.id as number, installPluginModal, configPayload, privateKey);

    if (result.success) {
        toast.success(`${installPluginModal} installed and configured securely! Node is restarting.`, { id: 'plugin' });
        setInstallPluginModal(null);
        setPrivateKey('');
        setPluginConfigInput('');
    } else {
        toast.error(result.error || 'Failed to install plugin.', { id: 'plugin' });
    }
    setIsInstalling(false);
  };

  const executeRemovePlugin = async (pluginId: string) => {
    if (!endpoint?.id) return;

    setIsRemovingPlugin(pluginId);
    const result = await removeNodePluginAction(Number(endpoint.id), pluginId);
    if (result.success) {
      toast.success(`${pluginId} removed successfully.`, { id: 'plugin-remove' });
      setPluginState((current) => current.filter((plugin) => plugin.pluginId !== pluginId));
      router.refresh();
      return;
    }

    toast.error(result.error || 'Failed to remove plugin.', { id: 'plugin-remove' });
    setIsRemovingPlugin(null);
  };

  const handleRemovePlugin = async (pluginId: string) => {
    setPendingConfirmation({
      kind: 'remove-plugin',
      pluginId,
      title: `Remove ${pluginId}?`,
      description: 'This will remove the plugin configuration from the managed node and delete the tracked plugin state from NeoNexus.',
      confirmLabel: 'Remove Plugin',
    });
  };

  const handleViewPluginLogs = async (pluginId: string) => {
    if (!endpoint?.id) return;
    setIsLoadingPluginLogs(true);
    setPluginLogsModal(pluginId);

    const response = await fetch(`/api/endpoints/${endpoint.id}/plugins/${pluginId}/logs`);
    const data = await response.json() as { logs?: string; error?: string };

    if (!response.ok) {
      setPluginLogs(data.error || 'Failed to fetch plugin logs.');
      setIsLoadingPluginLogs(false);
      return;
    }

    setPluginLogs(data.logs || '');
    setIsLoadingPluginLogs(false);
  };

  const handleCreateAlert = async () => {
    if (!endpoint?.id || !newAlertTarget.trim()) {
      toast.error('Provide a valid target for the alert.');
      return;
    }

    setIsCreatingAlert(true);
    const result = await createAlertRuleAction({
      endpointId: Number(endpoint.id),
      condition: newAlertCondition,
      actionType: newAlertActionType,
      target: newAlertTarget.trim(),
    });

    if (result.success) {
      toast.success('Alert rule created.');
      setNewAlertTarget('');
      const response = await fetch(`/api/endpoints/${endpoint.id}/alerts`);
      if (response.ok) {
        const data = await response.json() as {
          rules: AlertRuleView[];
          incidents: AlertIncidentView[];
        };
        setAlerts(data.rules);
        setAlertIncidents(data.incidents);
      }
    } else {
      toast.error(result.error || 'Failed to create alert rule.');
    }

    setIsCreatingAlert(false);
  };

  const handleDeleteAlert = async (alertRuleId: number) => {
    if (!endpoint?.id) return;

    setDeletingAlertId(alertRuleId);
    const result = await deleteAlertRuleAction(alertRuleId, Number(endpoint.id));
    if (result.success) {
      toast.success('Alert rule deleted.');
      setAlerts((current) => current.filter((alert) => alert.id !== alertRuleId));
    } else {
      toast.error(result.error || 'Failed to delete alert rule.');
    }
    setDeletingAlertId(null);
  };

  const handleSaveSettings = async () => {
    if (!endpoint?.id) return;

    setIsSavingSettings(true);
    const result = await updateEndpointSettingsAction(Number(endpoint.id), settingsForm);
    if (result.success) {
      toast.success('Node settings updated. The node is restarting to apply changes.', { id: 'settings' });
      router.refresh();
      return;
    }

    toast.error(result.error || 'Failed to update node settings.', { id: 'settings' });
    setIsSavingSettings(false);
  };

  const executeResync = async () => {
    if (!endpoint?.id) return;

    setMaintenanceAction('resync');
    const result = await triggerEndpointResyncAction(Number(endpoint.id));
    if (result.success) {
      toast.success('Resync started. The node is rebuilding from its bootstrap flow.', { id: 'maint' });
      router.refresh();
      return;
    }

    toast.error(result.error || 'Failed to trigger resync.', { id: 'maint' });
    setMaintenanceAction(null);
  };

  const handleResync = async () => {
    setPendingConfirmation({
      kind: 'resync',
      title: 'Trigger fast resync?',
      description: 'This purges the local chain data on the managed node and rebuilds it from the bootstrap flow. Use it only when the node needs a full resync.',
      confirmLabel: 'Start Resync',
    });
  };

  const handleExportSnapshot = async () => {
    if (!endpoint?.id) return;

    setMaintenanceAction('snapshot');
    const result = await exportEndpointSnapshotAction(Number(endpoint.id));
    if (result.success) {
      toast.success(`Snapshot exported on the node at ${result.snapshotPath}.`, { id: 'maint' });
    } else {
      toast.error(result.error || 'Failed to export snapshot.', { id: 'maint' });
    }
    setMaintenanceAction(null);
  };

  const handleRestart = async () => {
    if (!endpoint?.id) return;
    setIsLifecycleLoading('restart');
    const result = await restartEndpointAction(Number(endpoint.id));
    if (result.success) {
      toast.success('Restart requested. Endpoint status is syncing.', { id: 'lifecycle' });
      router.refresh();
    } else {
      toast.error(result.error || 'Failed to restart endpoint.', { id: 'lifecycle' });
    }
    setIsLifecycleLoading(null);
  };

  const handleStop = async () => {
    if (!endpoint?.id) return;
    setIsLifecycleLoading('stop');
    const result = await stopEndpointAction(Number(endpoint.id));
    if (result.success) {
      toast.success('Endpoint stop requested.', { id: 'lifecycle' });
      router.refresh();
    } else {
      toast.error(result.error || 'Failed to stop endpoint.', { id: 'lifecycle' });
    }
    setIsLifecycleLoading(null);
  };

  const executeDestroy = async () => {
    if (!endpoint?.id) return;

    setIsLifecycleLoading('destroy');
    const result = await destroyEndpointAction(Number(endpoint.id));
    if (result.success) {
      toast.success('Endpoint destroyed.', { id: 'lifecycle' });
      router.push('/app/endpoints');
      router.refresh();
      return;
    }

    toast.error(result.error || 'Failed to destroy endpoint.', { id: 'lifecycle' });
    setIsLifecycleLoading(null);
  };

  const handleDestroy = async () => {
    setPendingConfirmation({
      kind: 'destroy',
      title: 'Destroy this endpoint permanently?',
      description: 'This removes the route, deletes the endpoint record, and destroys the dedicated node footprint. This action cannot be undone.',
      confirmLabel: 'Destroy Endpoint',
    });
  };

  const handleConfirmAction = async () => {
    if (!pendingConfirmation) {
      return;
    }

    const confirmation = pendingConfirmation;
    setPendingConfirmation(null);

    if (confirmation.kind === 'remove-plugin') {
      await executeRemovePlugin(confirmation.pluginId);
      return;
    }

    if (confirmation.kind === 'resync') {
      await executeResync();
      return;
    }

    await executeDestroy();
  };

  const tabs = [
    { id: 'overview', name: 'Overview' },
    { id: 'metrics', name: 'Health Metrics' },
    { id: 'logs', name: 'Node Logs' },
    { id: 'plugins', name: 'Plugins' },
    { id: 'alerts', name: 'Alerts' },
    { id: 'activity', name: 'Activity' },
    { id: 'settings', name: 'Settings' },
  ];

  const connectionDisplay = deriveEndpointConnectionDisplay({
    status: endpoint?.status,
    url: endpoint?.url,
    wssUrl: endpoint?.wssUrl,
  });
  const installedPlugins = new Map(pluginState.map((plugin) => [plugin.pluginId, plugin]));
  const supportedNodeSettingKeys = endpoint?.clientEngine
    ? getSupportedNodeSettingKeys(endpoint.clientEngine as 'neo-go' | 'neo-cli' | 'neo-x-geth')
    : ['maxPeers', 'rpcEnabled'];
  const supportsMaxPeers = supportedNodeSettingKeys.includes('maxPeers');
  const supportsRpc = supportedNodeSettingKeys.includes('rpcEnabled');
  const supportsWebsocket = supportedNodeSettingKeys.includes('websocketEnabled');
  const supportsGraphql = supportedNodeSettingKeys.includes('graphqlEnabled');
  const supportsCache = supportedNodeSettingKeys.includes('cacheMb');

  const formatBytes = (value: number | null) => {
    if (value === null || Number.isNaN(value)) return 'N/A';
    const units = ['B', 'KB', 'MB', 'GB', 'TB'];
    let current = value;
    let unitIndex = 0;
    while (current >= 1024 && unitIndex < units.length - 1) {
      current /= 1024;
      unitIndex += 1;
    }
    return `${current.toFixed(unitIndex === 0 ? 0 : 1)} ${units[unitIndex]}`;
  };


  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <Link href="/app/endpoints" className="inline-flex items-center text-sm text-gray-400 hover:text-white mb-4 transition-colors">
          <ArrowLeft className="w-4 h-4 mr-1" /> Back to Endpoints
        </Link>
        <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
          <div>
            <h1 className="text-2xl font-bold tracking-tight text-white flex items-center gap-3">
              {endpoint?.name || 'GameFi Indexer Node'}
              <span className={`text-xs font-medium px-2 py-1 rounded-full border ${
                endpoint?.status.toLowerCase() === 'active' 
                  ? 'border-[#00E599]/30 text-[#00E599] bg-[#00E599]/10' 
                  : 'border-yellow-500/30 text-yellow-500 bg-yellow-500/10'
              }`}>
                {endpoint?.status || 'Active'}
              </span>
            </h1>
            <p className="text-sm text-gray-400 mt-1 flex items-center gap-2">
              <Server className="w-4 h-4" /> {endpoint?.type || 'Dedicated'} {endpoint?.network || 'N3 Mainnet'} • {endpoint?.cloudProvider || 'Hetzner'} {endpoint?.region || ''}
            </p>
            {orderState && endpoint?.status.toLowerCase() !== 'active' && (
              <p className="text-xs text-gray-500 mt-2">
                Provisioning step: <span className="text-[#00E599]">{orderState.currentStep}</span>
                {orderState.provider ? ` via ${orderState.provider}` : ''}
                {orderState.attemptCount ? ` • attempt ${orderState.attemptCount}` : ''}
                {orderState.errorMessage ? ` • ${orderState.errorMessage}` : ''}
                {orderState.nextAttemptAt ? ` • retry at ${new Date(orderState.nextAttemptAt).toLocaleTimeString()}` : ''}
              </p>
            )}
          </div>
          
          <div className="flex items-center gap-3">
            <button
              onClick={handleRestart}
              disabled={isLifecycleLoading !== null}
              className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] hover:bg-[#252525] disabled:opacity-50 text-white px-4 py-2 rounded-md text-sm font-medium transition-colors flex items-center gap-2"
            >
              <RotateCcw className="w-4 h-4" /> {isLifecycleLoading === 'restart' ? 'Restarting...' : 'Restart'}
            </button>
            <button
              onClick={handleStop}
              disabled={isLifecycleLoading !== null}
              className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] hover:bg-[#252525] disabled:opacity-50 text-white px-4 py-2 rounded-md text-sm font-medium transition-colors flex items-center gap-2"
            >
              <Power className="w-4 h-4" /> {isLifecycleLoading === 'stop' ? 'Stopping...' : 'Stop'}
            </button>
            <button
              onClick={handleDestroy}
              disabled={isLifecycleLoading !== null}
              className="bg-red-500/10 border border-red-500/20 hover:bg-red-500/20 disabled:opacity-50 text-red-300 px-4 py-2 rounded-md text-sm font-medium transition-colors flex items-center gap-2"
            >
              <MoreVertical className="w-4 h-4" /> {isLifecycleLoading === 'destroy' ? 'Destroying...' : 'Destroy'}
            </button>
          </div>
        </div>
      </div>

      {/* Tabs */}
      <div className="border-b border-[var(--color-dark-border)]">
        <nav className="flex space-x-8">
          {tabs.map((tab) => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={`py-4 text-sm font-medium border-b-2 transition-colors ${
                activeTab === tab.id
                  ? 'border-[#00E599] text-[#00E599]'
                  : 'border-transparent text-gray-400 hover:text-white hover:border-gray-600'
              }`}
            >
              {tab.name}
            </button>
          ))}
        </nav>
      </div>

      {/* Tab Content */}
      <div className="pt-2">
        {activeTab === 'overview' && (
          <div className="space-y-6 animate-in fade-in">
            {/* Connection URLs */}
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl p-6">
                <div className="flex items-center gap-2 mb-4">
                  <Globe className="w-5 h-5 text-blue-400" />
                  <h2 className="text-lg font-medium text-white">HTTPS Endpoint</h2>
                  <span className={`text-[10px] font-bold uppercase tracking-wide px-2 py-1 rounded ${
                    connectionDisplay.readiness === 'live'
                      ? 'bg-[#00E599]/10 text-[#00E599]'
                      : connectionDisplay.readiness === 'planned'
                        ? 'bg-yellow-500/10 text-yellow-300'
                        : 'bg-red-500/10 text-red-300'
                  }`}>
                    {connectionDisplay.label}
                  </span>
                </div>
                <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] p-3 rounded-lg flex items-center justify-between group">
                  {connectionDisplay.httpsUrl ? (
                    <>
                      <code className="text-sm text-[#00E599] truncate">{connectionDisplay.httpsUrl}</code>
                      <button 
                        onClick={() => navigator.clipboard.writeText(connectionDisplay.httpsUrl ?? '')}
                        className="text-gray-500 hover:text-white p-1 rounded transition-colors bg-[var(--color-dark-panel)] opacity-0 group-hover:opacity-100"
                      >
                        <Copy className="w-4 h-4" />
                      </button>
                    </>
                  ) : (
                    <span className="text-sm text-gray-500">Available after route configuration completes.</span>
                  )}
                </div>
              </div>

              <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl p-6">
                <div className="flex items-center gap-2 mb-4">
                  <Activity className="w-5 h-5 text-purple-400" />
                  <h2 className="text-lg font-medium text-white">WSS Endpoint (WebSocket)</h2>
                  <span className={`text-[10px] font-bold uppercase tracking-wide px-2 py-1 rounded ${
                    connectionDisplay.readiness === 'live'
                      ? 'bg-[#00E599]/10 text-[#00E599]'
                      : connectionDisplay.readiness === 'planned'
                        ? 'bg-yellow-500/10 text-yellow-300'
                        : 'bg-red-500/10 text-red-300'
                  }`}>
                    {connectionDisplay.label}
                  </span>
                </div>
                <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] p-3 rounded-lg flex items-center justify-between group">
                  {connectionDisplay.wssUrl ? (
                    <>
                      <code className="text-sm text-purple-400 truncate">{connectionDisplay.wssUrl}</code>
                      <button 
                        onClick={() => navigator.clipboard.writeText(connectionDisplay.wssUrl ?? '')}
                        className="text-gray-500 hover:text-white p-1 rounded transition-colors bg-[var(--color-dark-panel)] opacity-0 group-hover:opacity-100"
                      >
                        <Copy className="w-4 h-4" />
                      </button>
                    </>
                  ) : (
                    <span className="text-sm text-gray-500">WebSocket routing is not configured yet.</span>
                  )}
                </div>
              </div>
            </div>

            {/* Quick Stats */}
            <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl p-6">
              <h2 className="text-lg font-medium text-white mb-6">Node Status</h2>
              <div className="grid grid-cols-2 md:grid-cols-4 gap-6">
                <div>
                  <div className="text-sm text-gray-400 mb-1 flex items-center gap-1"><Box className="w-3 h-3"/> Block Height</div>
                  <div className="text-2xl font-bold text-white">{blockHeight}</div>
                </div>
                <div>
                  <div className="text-sm text-gray-400 mb-1 flex items-center gap-1"><Globe className="w-3 h-3"/> Peers Connected</div>
                  <div className="text-2xl font-bold text-white">{peerCount}</div>
                </div>
                <div>
                  <div className="text-sm text-gray-400 mb-1 flex items-center gap-1"><Terminal className="w-3 h-3"/> Client Engine</div>
                  <div className="text-2xl font-bold text-white">{endpoint?.clientEngine || 'neo-go'}</div>
                </div>
                <div>
                  <div className="text-sm text-gray-400 mb-1 flex items-center gap-1"><Activity className="w-3 h-3"/> Sync Mode</div>
                  <div className="text-2xl font-bold text-white uppercase">{endpoint?.syncMode || 'FULL'}</div>
                </div>
              </div>
            </div>
          </div>
        )}

        {activeTab === 'metrics' && (
          <div className="space-y-6 animate-in fade-in">
            {nodeMetricsError ? (
              <div className="bg-[var(--color-dark-panel)] border border-red-500/30 rounded-xl p-6 text-sm text-red-300">
                {nodeMetricsError}
              </div>
            ) : (
              <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
                <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl p-6">
                  <div className="flex justify-between items-center mb-4">
                    <h2 className="text-lg font-medium text-white flex items-center gap-2"><Cpu className="w-5 h-5 text-blue-400" /> CPU Usage</h2>
                    <span className="text-sm font-bold text-white">{nodeMetrics?.cpuPercent?.toFixed(1) ?? 'N/A'}%</span>
                  </div>
                  <p className="text-sm text-gray-400">Current Docker CPU percentage for the managed node container.</p>
                </div>

                <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl p-6">
                  <div className="flex justify-between items-center mb-4">
                    <h2 className="text-lg font-medium text-white flex items-center gap-2"><Activity className="w-5 h-5 text-[#00E599]" /> Memory Usage</h2>
                    <span className="text-sm font-bold text-white">
                      {formatBytes(nodeMetrics?.memoryUsedBytes ?? null)} / {formatBytes(nodeMetrics?.memoryLimitBytes ?? null)}
                    </span>
                  </div>
                  <p className="text-sm text-gray-400">Current Docker memory usage for the managed node container.</p>
                </div>

                <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl p-6">
                  <div className="flex justify-between items-center mb-4">
                    <h2 className="text-lg font-medium text-white flex items-center gap-2"><Globe className="w-5 h-5 text-purple-400" /> Network I/O</h2>
                    <span className="text-sm font-bold text-white">
                      {formatBytes(nodeMetrics?.networkRxBytes ?? null)} / {formatBytes(nodeMetrics?.networkTxBytes ?? null)}
                    </span>
                  </div>
                  <p className="text-sm text-gray-400">Receive / transmit totals reported by Docker stats.</p>
                </div>

                <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl p-6">
                  <div className="flex justify-between items-center mb-4">
                    <h2 className="text-lg font-medium text-white flex items-center gap-2"><HardDrive className="w-5 h-5 text-yellow-400" /> Block I/O</h2>
                    <span className="text-sm font-bold text-white">
                      {formatBytes(nodeMetrics?.blockReadBytes ?? null)} / {formatBytes(nodeMetrics?.blockWriteBytes ?? null)}
                    </span>
                  </div>
                  <p className="text-sm text-gray-400">Read / write totals reported by Docker for the node container.</p>
                </div>

                <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl p-6 lg:col-span-2">
                  <div className="flex justify-between items-center mb-4">
                    <h2 className="text-lg font-medium text-white flex items-center gap-2"><HardDrive className="w-5 h-5 text-pink-400" /> Disk Usage</h2>
                    <span className="text-sm font-bold text-white">
                      {formatBytes(nodeMetrics?.diskUsedBytes ?? null)} / {formatBytes(nodeMetrics?.diskTotalBytes ?? null)}
                    </span>
                  </div>
                  <p className="text-sm text-gray-400">
                    Disk consumption for the node data directory on the managed VM.
                    {nodeMetrics?.updatedAt ? ` Updated ${new Date(nodeMetrics.updatedAt).toLocaleTimeString()}.` : ''}
                  </p>
                </div>
              </div>
            )}
          </div>
        )}

        {activeTab === 'logs' && (
          <div className="animate-in fade-in">
            <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl overflow-hidden font-mono text-sm">
              <div className="bg-[var(--color-dark-panel)] border-b border-[var(--color-dark-border)] px-4 py-2 flex justify-between items-center">
                <div className="flex space-x-2">
                  <div className="w-3 h-3 rounded-full bg-red-500"></div>
                  <div className="w-3 h-3 rounded-full bg-yellow-500"></div>
                  <div className="w-3 h-3 rounded-full bg-green-500"></div>
                </div>
                <span className="text-gray-400 text-xs flex items-center gap-1"><Activity className="w-3 h-3 text-[#00E599]" /> {isLoadingNodeLogs ? 'Refreshing…' : 'Live Stream'}</span>
              </div>
              <div className="p-4 h-96 overflow-y-auto text-gray-300 whitespace-pre-wrap">
                {nodeLogs}
              </div>
            </div>
          </div>
        )}

        {activeTab === 'plugins' && (
          <div className="animate-in fade-in space-y-4">
            {listSupportedPlugins().map((pluginDefinition) => {
              const installed = installedPlugins.get(pluginDefinition.id);
              const isInstalled = Boolean(installed);
              const isError = installed?.status === 'Error';
              const isPending = installed?.status === 'Installing' || installed?.status === 'Pending Apply';
              const badgeClass = isError
                ? 'bg-red-500/20 text-red-300 border border-red-500/30'
                : isPending
                  ? 'bg-yellow-500/20 text-yellow-300 border border-yellow-500/30'
                  : 'bg-[#00E599]/20 text-[#00E599] border border-[#00E599]/30';

              return (
                <div
                  key={pluginDefinition.id}
                  className={`bg-[var(--color-dark-panel)] rounded-xl p-6 flex flex-col md:flex-row gap-6 justify-between items-start md:items-center border ${
                    isInstalled ? 'border-[#00E599]/30' : 'border-[var(--color-dark-border)]'
                  }`}
                >
                  <div className="flex gap-4">
                    <div className={`p-3 rounded-lg shrink-0 ${isInstalled ? 'bg-[#00E599]/10' : 'bg-gray-800'}`}>
                      {pluginDefinition.category === 'Security' ? (
                        <Lock className={`w-6 h-6 ${isInstalled ? 'text-[#00E599]' : 'text-gray-400'}`} />
                      ) : (
                        <Plug className={`w-6 h-6 ${isInstalled ? 'text-[#00E599]' : 'text-gray-400'}`} />
                      )}
                    </div>
                    <div>
                      <h3 className="text-lg font-medium text-white flex items-center gap-2">
                        {pluginDefinition.name}
                        {isInstalled && (
                          <span className={`text-xs px-2 py-0.5 rounded ${badgeClass}`}>
                            {installed?.status}
                          </span>
                        )}
                      </h3>
                      <p className="text-sm text-gray-400 mt-1">{pluginDefinition.description}</p>
                      {installed?.errorMessage && (
                        <p className="text-xs text-red-300 mt-2">{installed.errorMessage}</p>
                      )}
                    </div>
                  </div>
                  <button
                    onClick={() => setInstallPluginModal(pluginDefinition.id)}
                    className={`${isInstalled ? 'bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] hover:bg-[#252525] text-white' : 'text-[#00E599] bg-[#00E599]/10 border border-[#00E599]/20 hover:bg-[#00E599]/20'} px-4 py-2 rounded text-sm transition-colors`}
                  >
                    {isInstalled ? 'Reconfigure' : 'Install Plugin'}
                  </button>
                  {isInstalled && (
                    <div className="flex flex-col gap-2">
                      <button
                        onClick={() => handleViewPluginLogs(pluginDefinition.id)}
                        disabled={isLoadingPluginLogs}
                        className="text-blue-200 bg-blue-500/10 border border-blue-500/20 hover:bg-blue-500/20 disabled:opacity-50 px-4 py-2 rounded text-sm transition-colors"
                      >
                        View Logs
                      </button>
                      <button
                        onClick={() => handleRemovePlugin(pluginDefinition.id)}
                        disabled={isRemovingPlugin !== null}
                        className="text-red-300 bg-red-500/10 border border-red-500/20 hover:bg-red-500/20 disabled:opacity-50 px-4 py-2 rounded text-sm transition-colors"
                      >
                        {isRemovingPlugin === pluginDefinition.id ? 'Removing...' : 'Remove'}
                      </button>
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        )}

        {activeTab === 'alerts' && (
          <div className="animate-in fade-in space-y-6">
            <div className="flex justify-between items-center">
              <div>
                <h2 className="text-lg font-medium text-white">Alert Rules</h2>
                <p className="text-sm text-gray-400 mt-1">Get notified when your node requires attention.</p>
              </div>
            </div>

            <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl p-6 space-y-4">
              <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                <select
                  value={newAlertCondition}
                  onChange={(event) => setNewAlertCondition(event.target.value)}
                  className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-md px-4 py-2 text-white"
                >
                  {listAlertRuleDefinitions().map((definition) => (
                    <option key={definition.id} value={definition.id}>
                      {definition.name}
                    </option>
                  ))}
                </select>
                <select
                  value={newAlertActionType}
                  onChange={(event) => setNewAlertActionType(event.target.value as 'email' | 'webhook')}
                  className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-md px-4 py-2 text-white"
                >
                  <option value="email">Email</option>
                  <option value="webhook">Webhook</option>
                </select>
                <input
                  value={newAlertTarget}
                  onChange={(event) => setNewAlertTarget(event.target.value)}
                  placeholder={newAlertActionType === 'email' ? 'alerts@example.com' : 'https://hooks.example.com'}
                  className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-md px-4 py-2 text-white"
                />
              </div>
              <button
                onClick={handleCreateAlert}
                disabled={isCreatingAlert}
                className="bg-[#00E599] hover:bg-[#00cc88] disabled:opacity-50 text-black px-4 py-2 rounded-md text-sm font-bold transition-colors"
              >
                {isCreatingAlert ? 'Creating...' : 'Create Alert'}
              </button>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              {alerts.length === 0 ? (
                <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl p-6 text-sm text-gray-400">
                  No alert rules configured for this endpoint yet.
                </div>
              ) : (
                alerts.map((alert) => (
                  <div key={alert.id} className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl p-6 relative overflow-hidden">
                    <div className="flex justify-between items-start mb-4">
                      <div className="flex items-center gap-3">
                        <div className={`p-2 rounded-lg ${alert.triggered ? 'bg-red-500/10' : 'bg-[#00E599]/10'}`}>
                          {alert.actionType === 'email' ? (
                            <Mail className={`w-5 h-5 ${alert.triggered ? 'text-red-400' : 'text-[#00E599]'}`} />
                          ) : (
                            <Webhook className={`w-5 h-5 ${alert.triggered ? 'text-red-400' : 'text-[#00E599]'}`} />
                          )}
                        </div>
                        <h3 className="font-bold text-white">{alert.name}</h3>
                      </div>
                      <span className={`text-xs px-2 py-1 rounded font-bold ${alert.triggered ? 'bg-red-500/20 text-red-300' : 'bg-[#00E599]/20 text-[#00E599]'}`}>
                        {alert.triggered ? 'Triggered' : 'Healthy'}
                      </span>
                    </div>
                    <p className="text-sm text-gray-400 mb-4">{alert.message}</p>
                    <div className="flex items-center gap-2 text-sm text-gray-300 bg-[var(--color-dark-panel)] p-2 rounded-md border border-[var(--color-dark-border)] mb-4">
                      {alert.actionType === 'email' ? <Mail className="w-4 h-4 text-gray-500" /> : <Webhook className="w-4 h-4 text-gray-500" />}
                      {alert.target}
                    </div>
                    <div className="text-xs text-gray-500 space-y-1 mb-4">
                      {alert.deliveryAttemptCount > 0 && <div>Delivery attempts: {alert.deliveryAttemptCount}</div>}
                      {alert.nextDeliveryAt && <div>Next retry: {new Date(alert.nextDeliveryAt).toLocaleString()}</div>}
                      {alert.lastDeliveredAt && <div>Last delivered: {new Date(alert.lastDeliveredAt).toLocaleString()}</div>}
                      {alert.lastResolvedAt && <div>Last resolved: {new Date(alert.lastResolvedAt).toLocaleString()}</div>}
                      {alert.lastDeliveryError && <div className="text-red-300">Delivery error: {alert.lastDeliveryError}</div>}
                    </div>
                    <button
                      onClick={() => handleDeleteAlert(alert.id)}
                      disabled={deletingAlertId !== null}
                      className="text-red-300 bg-red-500/10 border border-red-500/20 hover:bg-red-500/20 disabled:opacity-50 px-4 py-2 rounded text-sm transition-colors"
                    >
                      {deletingAlertId === alert.id ? 'Deleting...' : 'Delete Alert'}
                    </button>
                  </div>
                ))
              )}
            </div>

            <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl p-6">
              <h3 className="text-lg font-medium text-white mb-4">Recent Incidents</h3>
              {alertIncidents.length === 0 ? (
                <p className="text-sm text-gray-400">No alert incidents have been recorded yet.</p>
              ) : (
                <div className="space-y-3">
                  {alertIncidents.map((incident) => (
                    <div key={incident.id} className="border border-[var(--color-dark-border)] rounded-lg p-4">
                      <div className="flex items-center justify-between mb-2">
                        <span className={`text-xs px-2 py-1 rounded font-bold ${incident.status === 'Open' ? 'bg-red-500/20 text-red-300' : 'bg-[#00E599]/20 text-[#00E599]'}`}>
                          {incident.status}
                        </span>
                        <span className="text-xs text-gray-500">{new Date(incident.openedAt).toLocaleString()}</span>
                      </div>
                      <p className="text-sm text-white">{incident.message}</p>
                      <div className="text-xs text-gray-500 mt-2 space-y-1">
                        <div>Severity: {incident.severity}</div>
                        {incident.lastDeliveredAt && <div>Delivered: {new Date(incident.lastDeliveredAt).toLocaleString()}</div>}
                        {incident.resolvedAt && <div>Resolved: {new Date(incident.resolvedAt).toLocaleString()}</div>}
                        {incident.lastDeliveryError && <div className="text-red-300">Delivery error: {incident.lastDeliveryError}</div>}
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </div>
        )}

        {activeTab === 'activity' && (
          <div className="animate-in fade-in space-y-4">
            {activities.length === 0 ? (
              <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl p-6 text-sm text-gray-400">
                No endpoint activity has been recorded yet.
              </div>
            ) : (
              activities.map((activity) => (
                <div key={activity.id} className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl p-4">
                  <div className="flex items-center justify-between gap-4 mb-2">
                    <div className="flex items-center gap-2">
                      <span className={`text-xs px-2 py-1 rounded font-bold ${
                        activity.status === 'error'
                          ? 'bg-red-500/20 text-red-300'
                          : activity.status === 'success'
                            ? 'bg-[#00E599]/20 text-[#00E599]'
                            : 'bg-yellow-500/20 text-yellow-300'
                      }`}>
                        {activity.status}
                      </span>
                      <span className="text-xs uppercase tracking-wide text-gray-500">{activity.category}</span>
                      <span className="text-xs text-gray-500">{activity.action}</span>
                    </div>
                    <span className="text-xs text-gray-500">{new Date(activity.createdAt).toLocaleString()}</span>
                  </div>
                  <p className="text-sm text-gray-300">{activity.message}</p>
                  {activity.metadata !== null && activity.metadata !== undefined && (
                    (() => {
                      const metadataRows = formatEndpointActivityMetadata(activity.metadata);
                      return metadataRows.length > 0 ? (
                        <div className="mt-3 text-xs text-gray-500 bg-[var(--color-dark-bg)] border border-[var(--color-dark-border)] rounded-md p-3 space-y-1">
                          {metadataRows.map((row) => (
                            <div key={`${activity.id}-${row.label}`} className="flex items-center justify-between gap-4">
                              <span className="text-gray-400">{row.label}</span>
                              <span className="text-right break-all">{row.value}</span>
                            </div>
                          ))}
                        </div>
                      ) : (
                        <pre className="mt-3 text-xs text-gray-500 bg-[var(--color-dark-bg)] border border-[var(--color-dark-border)] rounded-md p-3 overflow-x-auto whitespace-pre-wrap">
                          {String(JSON.stringify(activity.metadata, null, 2))}
                        </pre>
                      );
                    })()
                  )}
                </div>
              ))
            )}
          </div>
        )}

        {activeTab === 'settings' && (
          <div className="animate-in fade-in space-y-6">
            <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl p-6">
              <div className="flex items-center gap-3 mb-4">
                 <h2 className="text-lg font-medium text-white">Node Configuration</h2>
                 <span className="bg-gray-800 text-gray-300 text-xs px-2 py-0.5 rounded font-mono">{endpoint?.clientEngine || 'neo-go'}</span>
              </div>
              <p className="text-sm text-gray-400 mb-6">Modify your core node parameters. Changes are persisted and pushed to the managed VM, then the node container is restarted.</p>
              <div className="space-y-5">
                {supportsMaxPeers && (
                  <div>
                    <label className="block text-sm font-medium text-gray-300 mb-2">
                      Max Connected Peers{endpoint?.clientEngine === 'neo-x-geth' ? ' (--maxpeers)' : ''}
                    </label>
                    <input type="number" value={settingsForm.maxPeers} onChange={(e) => setSettingsForm((current) => ({ ...current, maxPeers: Number.parseInt(e.target.value, 10) || 0 }))} className="w-full md:w-1/3 bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-md px-4 py-2 text-white focus:outline-none focus:border-[#00E599]" />
                  </div>
                )}

                {supportsCache && (
                  <div>
                    <label className="block text-sm font-medium text-gray-300 mb-2">EVM Cache Size (--cache)</label>
                    <input type="number" value={settingsForm.cacheMb ?? 0} onChange={(e) => setSettingsForm((current) => ({ ...current, cacheMb: Number.parseInt(e.target.value, 10) || 0 }))} className="w-full md:w-1/3 bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-md px-4 py-2 text-white focus:outline-none focus:border-[#00E599]" />
                    <p className="text-xs text-gray-500 mt-1">Megabytes of memory allocated to internal caching.</p>
                  </div>
                )}

                <div className="pt-4 space-y-4">
                  {supportsRpc && (
                    <label className="flex items-center gap-3 cursor-pointer">
                      <input type="checkbox" checked={settingsForm.rpcEnabled} onChange={(e) => setSettingsForm((current) => ({ ...current, rpcEnabled: e.target.checked }))} className="w-4 h-4 rounded border-[var(--color-dark-border)] bg-[var(--color-dark-panel)] text-[#00E599] focus:ring-[#00E599]" />
                      <span className="text-sm text-gray-300">{endpoint?.clientEngine === 'neo-x-geth' ? 'Enable HTTP RPC API (--http)' : 'Enable RPC API'}</span>
                    </label>
                  )}
                  {supportsWebsocket && (
                    <label className="flex items-center gap-3 cursor-pointer">
                      <input type="checkbox" checked={settingsForm.websocketEnabled} onChange={(e) => setSettingsForm((current) => ({ ...current, websocketEnabled: e.target.checked }))} className="w-4 h-4 rounded border-[var(--color-dark-border)] bg-[var(--color-dark-panel)] text-[#00E599] focus:ring-[#00E599]" />
                      <span className="text-sm text-gray-300">{endpoint?.clientEngine === 'neo-x-geth' ? 'Enable WebSocket API (--ws)' : 'Enable WebSocket API'}</span>
                    </label>
                  )}
                  {supportsGraphql && (
                    <label className="flex items-center gap-3 cursor-pointer">
                      <input type="checkbox" checked={settingsForm.graphqlEnabled} onChange={(e) => setSettingsForm((current) => ({ ...current, graphqlEnabled: e.target.checked }))} className="w-4 h-4 rounded border-[var(--color-dark-border)] bg-[var(--color-dark-panel)] text-[#00E599] focus:ring-[#00E599]" />
                      <span className="text-sm text-gray-300">{endpoint?.clientEngine === 'neo-x-geth' ? 'Enable GraphQL API (--graphql)' : 'Enable GraphQL API'}</span>
                    </label>
                  )}
                </div>

                {endpoint?.clientEngine !== 'neo-x-geth' && (
                  <p className="text-xs text-gray-500">
                    Only settings that map to the current managed runtime are exposed here. Advanced engine-specific tuning remains pinned to the bootstrap profile for now.
                  </p>
                )}
              </div>

              <div className="pt-6 mt-6 border-t border-[var(--color-dark-border)]">
                <button 
                  onClick={handleSaveSettings}
                  disabled={isSavingSettings}
                  className="bg-[#00E599] hover:bg-[#00cc88] disabled:opacity-50 text-black px-6 py-2 rounded-md font-bold transition-colors"
                >
                  {isSavingSettings ? 'Saving...' : 'Save Configuration'}
                </button>
              </div>
            </div>

            <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl p-6">
              <h2 className="text-lg font-medium text-white mb-4">Maintenance Actions</h2>
              <p className="text-sm text-gray-400 mb-6">Perform advanced lifecycle operations on your node storage and state.</p>
              
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <div className="border border-[var(--color-dark-border)] rounded-lg p-4 flex flex-col justify-between">
                   <div className="mb-4">
                     <h3 className="font-bold text-white flex items-center gap-2 mb-1"><RefreshCw className="w-4 h-4 text-blue-400" /> Fast Resync</h3>
                     <p className="text-xs text-gray-400">Purges local ledger data and automatically redownloads the latest official Neo Genesis snapshot.</p>
                   </div>
                  <button 
                     onClick={handleResync}
                     disabled={maintenanceAction !== null}
                     className="bg-[var(--color-dark-panel)] hover:bg-[#222222] disabled:opacity-50 border border-[var(--color-dark-border)] text-white px-4 py-2 rounded-md text-sm font-bold transition-colors w-full"
                   >
                     {maintenanceAction === 'resync' ? 'Resyncing...' : 'Trigger Resync'}
                   </button>
                </div>
                <div className="border border-[var(--color-dark-border)] rounded-lg p-4 flex flex-col justify-between">
                   <div className="mb-4">
                     <h3 className="font-bold text-white flex items-center gap-2 mb-1"><Download className="w-4 h-4 text-green-400" /> Export Snapshot</h3>
                     <p className="text-xs text-gray-400">Creates a point-in-time tarball of your Node&apos;s persistent volume and generates a signed S3 download link.</p>
                   </div>
                   <button 
                     onClick={handleExportSnapshot}
                     disabled={maintenanceAction !== null}
                     className="bg-[var(--color-dark-panel)] hover:bg-[#222222] disabled:opacity-50 border border-[var(--color-dark-border)] text-white px-4 py-2 rounded-md text-sm font-bold transition-colors w-full"
                   >
                     {maintenanceAction === 'snapshot' ? 'Exporting...' : 'Export Data'}
                   </button>
                </div>
              </div>
            </div>

            <div className="bg-red-500/10 border border-red-500/30 rounded-xl p-6">
              <h2 className="text-lg font-medium text-red-400 mb-2">Danger Zone</h2>
              <p className="text-sm text-gray-400 mb-4">Deleting this endpoint is permanent and cannot be undone. You will lose the dedicated IP, URL, and all local node data on the managed VM.</p>
              <button 
                onClick={handleDestroy}
                disabled={isLifecycleLoading !== null}
                className="bg-red-500 hover:bg-red-600 disabled:opacity-50 text-white px-6 py-2 rounded-md font-bold transition-colors"
              >
                {isLifecycleLoading === 'destroy' ? 'Destroying...' : 'Delete Endpoint'}
              </button>
            </div>
          </div>
        )}
      </div>

      {installPluginModal && (
        <div className="fixed inset-0 bg-black/80 backdrop-blur-sm z-50 flex items-center justify-center p-4">
          <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-2xl max-w-md w-full p-6 shadow-2xl relative">
            <button onClick={() => { setInstallPluginModal(null); setPrivateKey(''); setPluginConfigInput(''); }} className="absolute top-4 right-4 text-gray-500 hover:text-white">✕</button>
            <h3 className="text-xl font-bold text-white mb-2">
              Configure {installPluginModal === 'tee-oracle' ? 'TEE Oracle' : (installPluginModal === 'tee-mempool' ? 'TEE Protected Mempool' : 'AA Bundler')}
            </h3>
            <p className="text-sm text-gray-400 mb-6">
              This plugin requires a wallet to sign transactions on-chain. Provide the signing key and any deployment note you want persisted with the plugin configuration. Secrets are encrypted before storage.
            </p>
            
            <div className="space-y-4 mb-6">
              <div>
                <label className="block text-sm font-medium text-gray-400 mb-1">Neo N3 Private Key (WIF)</label>
                <input 
                  type="password" 
                  value={privateKey}
                  onChange={(e) => setPrivateKey(e.target.value)}
                  placeholder="L1K..." 
                  className="w-full bg-[var(--color-dark-bg)] border border-[var(--color-dark-border)] rounded-xl px-4 py-3 text-white focus:outline-none focus:border-[#00E599] transition-colors"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-400 mb-1">Plugin Configuration Note</label>
                <textarea
                  value={pluginConfigInput}
                  onChange={(e) => setPluginConfigInput(e.target.value)}
                  placeholder="Optional route, webhook, or deployment note..."
                  rows={3}
                  className="w-full bg-[var(--color-dark-bg)] border border-[var(--color-dark-border)] rounded-xl px-4 py-3 text-white focus:outline-none focus:border-[#00E599] transition-colors"
                />
              </div>
            </div>

            <button 
              onClick={handleInstallPlugin}
              disabled={isInstalling || !privateKey}
              className="w-full bg-[#00E599] hover:bg-[#00cc88] disabled:opacity-50 text-black py-3 rounded-xl font-bold transition-all shadow-[0_0_15px_rgba(0,229,153,0.2)]"
            >
              {isInstalling ? 'Installing...' : 'Save securely & Deploy'}
            </button>
            <p className="text-xs text-center text-gray-500 mt-4 flex items-center justify-center gap-1">
              <Lock className="w-3 h-3" /> Encrypted and stored via the NeoNexus vault layer
            </p>
          </div>
        </div>
      )}

      {pendingConfirmation && (
        <div className="fixed inset-0 bg-black/80 backdrop-blur-sm z-50 flex items-center justify-center p-4">
          <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-2xl max-w-md w-full p-6 shadow-2xl relative">
            <button onClick={() => setPendingConfirmation(null)} className="absolute top-4 right-4 text-gray-500 hover:text-white">✕</button>
            <h3 className="text-xl font-bold text-white mb-3">{pendingConfirmation.title}</h3>
            <p className="text-sm text-gray-400 mb-6 leading-relaxed">{pendingConfirmation.description}</p>
            <div className="flex gap-3">
              <button
                onClick={() => setPendingConfirmation(null)}
                className="flex-1 bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] hover:bg-[#252525] text-white py-3 rounded-xl font-bold transition-colors"
              >
                Cancel
              </button>
              <button
                onClick={handleConfirmAction}
                className="flex-1 bg-red-500 hover:bg-red-600 text-white py-3 rounded-xl font-bold transition-colors"
              >
                {pendingConfirmation.confirmLabel}
              </button>
            </div>
          </div>
        </div>
      )}

      {pluginLogsModal && (
        <div className="fixed inset-0 bg-black/80 backdrop-blur-sm z-50 flex items-center justify-center p-4">
          <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-2xl max-w-3xl w-full p-6 shadow-2xl relative">
            <button onClick={() => { setPluginLogsModal(null); setPluginLogs(''); }} className="absolute top-4 right-4 text-gray-500 hover:text-white">✕</button>
            <h3 className="text-xl font-bold text-white mb-4">Plugin Logs: {pluginLogsModal}</h3>
            <div className="bg-[var(--color-dark-bg)] border border-[var(--color-dark-border)] rounded-xl p-4 h-96 overflow-y-auto font-mono text-sm text-gray-300 whitespace-pre-wrap">
              {isLoadingPluginLogs ? 'Loading plugin logs...' : (pluginLogs || 'No logs returned.')}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
