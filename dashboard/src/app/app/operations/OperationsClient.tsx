'use client';

import Link from 'next/link';
import { useState } from 'react';
import useSWR from 'swr';
import toast from 'react-hot-toast';
import { buildOperationsQueueHref } from '@/services/reconciliation/OperationsQueueFilters';
import {
  retryAlertDeliveryAction,
  retryProvisioningOrderAction,
  runControlPlaneReconcileAction,
} from './actions';

const fetcher = async (url: string) => {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`Failed to fetch ${url}`);
  }

  return response.json();
};

type ReconcileRun = {
  id: number;
  source: string;
  status: string;
  provisioningChecked: number;
  provisioningResumed: number;
  alertingEndpointsEvaluated: number;
  metadata: {
    resumedOrderIds?: number[];
    alertEndpointIds?: number[];
  } | null;
  errorMessage: string | null;
  createdAt: string;
};

type OperationsSummaryPayload = {
  summary: {
    pendingProvisioningCount: number;
    dueProvisioningCount: number;
    alertRetriesCount: number;
    openIncidentsCount: number;
    latestSuccessfulRunAt: string | null;
    latestFailedRunAt: string | null;
    latestScheduledRunAt: string | null;
    latestScheduledRunStatus: 'success' | 'error' | null;
    latestScheduledSuccessAt: string | null;
    latestScheduledFailureAt: string | null;
    schedulerStatus: 'healthy' | 'degraded' | 'stale';
    schedulerIsStale: boolean;
  };
  pendingOrders: Array<{
    id: number;
    endpointId: number;
    endpointName: string;
    currentStep: string;
    attemptCount: number;
    nextAttemptAt: string | null;
    errorMessage: string | null;
  }>;
  alertRetries: Array<{
    id: number;
    endpointId: number;
    endpointName: string;
    name: string;
    nextDeliveryAt: string | null;
    deliveryAttemptCount: number;
    lastDeliveryError: string | null;
  }>;
};

export default function OperationsClient() {
  const [isRunning, setIsRunning] = useState(false);
  const [retryingOrderId, setRetryingOrderId] = useState<number | null>(null);
  const [retryingAlertId, setRetryingAlertId] = useState<number | null>(null);
  const [reconcileRunsLimit, setReconcileRunsLimit] = useState(10);
  const { data: reconcileRuns = [], mutate } = useSWR<ReconcileRun[]>(`/api/operations/reconcile?limit=${reconcileRunsLimit}`, fetcher, {
    refreshInterval: 15000,
    fallbackData: [],
  });
  const { data: operationsSummary } = useSWR<OperationsSummaryPayload>('/api/operations/summary', fetcher, {
    refreshInterval: 15000,
  });

  const handleRunNow = async () => {
    setIsRunning(true);
    const result = await runControlPlaneReconcileAction();

    if (result.ok) {
      toast.success('Control plane reconcile completed successfully.');
      await mutate();
    } else {
      toast.error(result.error || 'Control plane reconcile failed.');
      await mutate();
    }

    setIsRunning(false);
  };

  const refreshAll = async () => {
    await mutate();
  };

  const handleRetryOrder = async (orderId: number) => {
    setRetryingOrderId(orderId);
    const result = await retryProvisioningOrderAction(orderId);
    if (result.success) {
      toast.success('Provisioning retry requested.');
      await refreshAll();
    } else {
      toast.error(result.error || 'Failed to retry provisioning order.');
    }
    setRetryingOrderId(null);
  };

  const handleRetryAlert = async (alertRuleId: number) => {
    setRetryingAlertId(alertRuleId);
    const result = await retryAlertDeliveryAction(alertRuleId);
    if (result.success) {
      toast.success('Alert delivery retry requested.');
      await refreshAll();
    } else {
      toast.error(result.error || 'Failed to retry alert delivery.');
    }
    setRetryingAlertId(null);
  };

  const schedulerStatus = operationsSummary?.summary.schedulerStatus ?? 'stale';
  const schedulerTone = schedulerStatus === 'healthy'
    ? 'text-[#00E599]'
    : schedulerStatus === 'degraded'
      ? 'text-amber-300'
      : 'text-red-300';
  const schedulerLabel = schedulerStatus === 'healthy'
    ? 'Healthy'
    : schedulerStatus === 'degraded'
      ? 'Degraded'
      : 'Stale';
  const provisioningQueueHref = buildOperationsQueueHref('/app/operations/provisioning', {});
  const dueProvisioningQueueHref = buildOperationsQueueHref('/app/operations/provisioning', { timing: 'due' });
  const alertQueueHref = buildOperationsQueueHref('/app/operations/alerts', {});
  const incidentQueueHref = buildOperationsQueueHref('/app/operations/incidents', {});

  return (
    <div className="space-y-8">
      <div className="flex items-center justify-between gap-4">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">Operations</h1>
          <p className="text-gray-400 mt-1">Run and inspect control-plane reconciliation cycles.</p>
        </div>
        <button
          onClick={handleRunNow}
          disabled={isRunning}
          className="bg-[#00E599] hover:bg-[#00cc88] disabled:opacity-50 text-black px-4 py-2 rounded-md text-sm font-bold transition-colors"
        >
          {isRunning ? 'Running…' : 'Run Reconcile Now'}
        </button>
      </div>

      {operationsSummary && (
        <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-5 gap-4">
          <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl p-4">
            <div className="text-xs text-gray-500 uppercase tracking-wide mb-2">Pending Provisioning</div>
            <div className="text-2xl font-bold text-white">{operationsSummary.summary.pendingProvisioningCount}</div>
            <Link href={provisioningQueueHref} className="inline-flex mt-3 text-xs text-[#00E599] hover:text-[#00cc88] font-medium">
              View Queue
            </Link>
          </div>
          <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl p-4">
            <div className="text-xs text-gray-500 uppercase tracking-wide mb-2">Due Provisioning Retries</div>
            <div className="text-2xl font-bold text-white">{operationsSummary.summary.dueProvisioningCount}</div>
            <Link href={dueProvisioningQueueHref} className="inline-flex mt-3 text-xs text-[#00E599] hover:text-[#00cc88] font-medium">
              View Due Queue
            </Link>
          </div>
          <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl p-4">
            <div className="text-xs text-gray-500 uppercase tracking-wide mb-2">Alert Delivery Retries</div>
            <div className="text-2xl font-bold text-white">{operationsSummary.summary.alertRetriesCount}</div>
            <Link href={alertQueueHref} className="inline-flex mt-3 text-xs text-[#00E599] hover:text-[#00cc88] font-medium">
              View Queue
            </Link>
          </div>
          <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl p-4">
            <div className="text-xs text-gray-500 uppercase tracking-wide mb-2">Open Incidents</div>
            <div className="text-2xl font-bold text-white">{operationsSummary.summary.openIncidentsCount}</div>
            <Link href={incidentQueueHref} className="inline-flex mt-3 text-xs text-[#00E599] hover:text-[#00cc88] font-medium">
              View Incidents
            </Link>
          </div>
          <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl p-4">
            <div className="text-xs text-gray-500 uppercase tracking-wide mb-2">Scheduler</div>
            <div className={`text-2xl font-bold ${schedulerTone}`}>
              {schedulerLabel}
            </div>
            {operationsSummary.summary.latestScheduledRunAt && (
              <p className="text-xs text-gray-500 mt-2">
                Last heartbeat: {new Date(operationsSummary.summary.latestScheduledRunAt).toLocaleString()}
              </p>
            )}
            {operationsSummary.summary.latestScheduledRunStatus === 'error' && operationsSummary.summary.latestScheduledFailureAt && (
              <p className="text-xs text-amber-300 mt-1">
                Last failure: {new Date(operationsSummary.summary.latestScheduledFailureAt).toLocaleString()}
              </p>
            )}
          </div>
        </div>
      )}

      <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl p-6">
        <div className="flex items-center justify-between gap-4 mb-4">
          <h2 className="text-lg font-medium text-white">Recent Reconcile Runs</h2>
          <div className="flex items-center gap-2 text-xs text-gray-400">
            <span>Show</span>
            <select
              value={reconcileRunsLimit}
              onChange={(event) => setReconcileRunsLimit(Number.parseInt(event.target.value, 10))}
              className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-md px-2 py-1 text-white"
            >
              <option value={10}>10</option>
              <option value={25}>25</option>
              <option value={50}>50</option>
            </select>
          </div>
        </div>
        {reconcileRuns.length === 0 ? (
          <p className="text-sm text-gray-400">No reconcile runs recorded yet.</p>
        ) : (
          <div className="space-y-3">
            {reconcileRuns.map((run) => (
              <div key={run.id} className="border border-[var(--color-dark-border)] rounded-lg p-4">
                <div className="flex items-center justify-between mb-2">
                  <span className={`text-xs px-2 py-1 rounded font-bold ${run.status === 'success' ? 'bg-[#00E599]/20 text-[#00E599]' : 'bg-red-500/20 text-red-300'}`}>
                    {run.status}
                  </span>
                  <span className="text-xs text-gray-500">{new Date(run.createdAt).toLocaleString()}</span>
                </div>
                <p className="text-xs text-gray-400">
                  Source: {run.source} • Provisioning checked: {run.provisioningChecked}, resumed: {run.provisioningResumed}, alerts evaluated: {run.alertingEndpointsEvaluated}
                </p>
                {run.metadata?.resumedOrderIds?.length ? (
                  <p className="text-xs text-gray-500 mt-2">Resumed orders: {run.metadata.resumedOrderIds.join(', ')}</p>
                ) : null}
                {run.metadata?.alertEndpointIds?.length ? (
                  <p className="text-xs text-gray-500 mt-1">Alert endpoints: {run.metadata.alertEndpointIds.join(', ')}</p>
                ) : null}
                {run.errorMessage && <p className="text-xs text-red-300 mt-2">{run.errorMessage}</p>}
                <Link href={`/app/operations/${run.id}`} className="inline-flex mt-3 text-xs text-[#00E599] hover:text-[#00cc88] font-medium">
                  View Run Details
                </Link>
              </div>
            ))}
          </div>
        )}
      </div>

      {operationsSummary && (
        <div className="grid grid-cols-1 xl:grid-cols-2 gap-6">
          <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl p-6">
            <div className="flex items-center justify-between gap-4 mb-4">
              <h2 className="text-lg font-medium text-white">Pending Provisioning Queue</h2>
              <Link href={provisioningQueueHref} className="text-xs text-[#00E599] hover:text-[#00cc88] font-medium">
                Open Full Queue
              </Link>
            </div>
            {operationsSummary.pendingOrders.length === 0 ? (
              <p className="text-sm text-gray-400">No pending provisioning orders.</p>
            ) : (
              <div className="space-y-3">
                {operationsSummary.pendingOrders.map((order) => (
                  <div key={order.id} className="border border-[var(--color-dark-border)] rounded-lg p-4">
                    <div className="flex items-center justify-between mb-2">
                      <Link href={`/app/endpoints/${order.endpointId}`} className="text-sm font-medium text-white hover:text-[#00E599]">
                        {order.endpointName}
                      </Link>
                      <span className="text-xs text-gray-500">Order #{order.id}</span>
                    </div>
                    <p className="text-xs text-gray-400">Step: {order.currentStep} • Attempts: {order.attemptCount}</p>
                    {order.nextAttemptAt && <p className="text-xs text-gray-500 mt-1">Next retry: {new Date(order.nextAttemptAt).toLocaleString()}</p>}
                    {order.errorMessage && <p className="text-xs text-red-300 mt-2">{order.errorMessage}</p>}
                    <div className="mt-3 flex flex-wrap gap-3">
                      <Link href={`/app/operations/provisioning/${order.id}`} className="text-xs text-[#00E599] hover:text-[#00cc88] font-medium">
                        View Details
                      </Link>
                      <button
                        onClick={() => handleRetryOrder(order.id)}
                        disabled={retryingOrderId !== null}
                        className="text-xs text-[#00E599] hover:text-[#00cc88] disabled:opacity-50 font-medium"
                      >
                        {retryingOrderId === order.id ? 'Retrying…' : 'Retry Now'}
                      </button>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>

          <div className="bg-[var(--color-dark-panel)] border border-[var(--color-dark-border)] rounded-xl p-6">
            <div className="flex items-center justify-between gap-4 mb-4">
              <h2 className="text-lg font-medium text-white">Alert Delivery Retry Queue</h2>
              <Link href={alertQueueHref} className="text-xs text-[#00E599] hover:text-[#00cc88] font-medium">
                Open Full Queue
              </Link>
            </div>
            {operationsSummary.alertRetries.length === 0 ? (
              <p className="text-sm text-gray-400">No pending alert delivery retries.</p>
            ) : (
              <div className="space-y-3">
                {operationsSummary.alertRetries.map((rule) => (
                  <div key={rule.id} className="border border-[var(--color-dark-border)] rounded-lg p-4">
                    <div className="flex items-center justify-between mb-2">
                      <Link href={`/app/endpoints/${rule.endpointId}`} className="text-sm font-medium text-white hover:text-[#00E599]">
                        {rule.endpointName}
                      </Link>
                      <span className="text-xs text-gray-500">{rule.name}</span>
                    </div>
                    <p className="text-xs text-gray-400">Attempts: {rule.deliveryAttemptCount}</p>
                    {rule.nextDeliveryAt && <p className="text-xs text-gray-500 mt-1">Next retry: {new Date(rule.nextDeliveryAt).toLocaleString()}</p>}
                    {rule.lastDeliveryError && <p className="text-xs text-red-300 mt-2">{rule.lastDeliveryError}</p>}
                    <div className="mt-3 flex flex-wrap gap-3">
                      <Link href={`/app/operations/alerts/${rule.id}`} className="text-xs text-[#00E599] hover:text-[#00cc88] font-medium">
                        View Details
                      </Link>
                      <button
                        onClick={() => handleRetryAlert(rule.id)}
                        disabled={retryingAlertId !== null}
                        className="text-xs text-[#00E599] hover:text-[#00cc88] disabled:opacity-50 font-medium"
                      >
                        {retryingAlertId === rule.id ? 'Retrying…' : 'Retry Now'}
                      </button>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
