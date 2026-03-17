import { reconcileControlPlane } from './ControlPlaneReconciler';
import { recordReconcileRun } from './ReconcileRunService';

export async function executeAndRecordReconcile(
  source: 'manual' | 'scheduled' = 'manual',
  reconcile = reconcileControlPlane,
  record = recordReconcileRun,
) {
  try {
    const result = await reconcile();
    const run = await record({
      source,
      status: 'success',
      provisioningChecked: result.provisioning.checked,
      provisioningResumed: result.provisioning.resumed,
      alertingEndpointsEvaluated: result.alerting.endpointsEvaluated,
      metadata: {
        resumedOrderIds: result.provisioning.resumedOrderIds,
        alertEndpointIds: result.alerting.endpointIds,
      },
    });

    return {
      ok: true as const,
      runId: run.id,
      result,
    };
  } catch (error) {
    const message = error instanceof Error ? error.message : 'Unknown reconciliation error.';
    const run = await record({
      source,
      status: 'error',
      provisioningChecked: 0,
      provisioningResumed: 0,
      alertingEndpointsEvaluated: 0,
      errorMessage: message,
    });

    return {
      ok: false as const,
      runId: run.id,
      error: message,
    };
  }
}
