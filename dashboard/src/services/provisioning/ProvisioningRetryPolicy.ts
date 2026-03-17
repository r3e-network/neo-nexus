export const MAX_PROVISIONING_ATTEMPTS = 5;

function getRetryDelayMs(attemptCount: number) {
  const minutes = Math.min(2 ** Math.max(attemptCount - 1, 0), 30);
  return minutes * 60_000;
}

export function buildProvisioningFailureUpdate(input: {
  attemptCount: number;
  errorMessage: string;
  now: Date;
}) {
  if (input.attemptCount >= MAX_PROVISIONING_ATTEMPTS) {
    return {
      status: 'failed' as const,
      currentStep: 'failed',
      errorMessage: input.errorMessage,
      nextAttemptAt: null,
    };
  }

  return {
    status: 'pending' as const,
    currentStep: 'pending',
    errorMessage: input.errorMessage,
    nextAttemptAt: new Date(input.now.getTime() + getRetryDelayMs(input.attemptCount)),
  };
}
