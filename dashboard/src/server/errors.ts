export function getErrorMessage(error: unknown, fallback = 'An unexpected error occurred'): string {
  if (error instanceof Error && error.message) {
    return error.message;
  }

  if (typeof error === 'string' && error.trim()) {
    return error;
  }

  return fallback;
}

type ErrorWithResponseBody = {
  response?: {
    body?: {
      message?: string;
    };
  };
};

export function getKubernetesErrorMessage(error: unknown, fallback = 'Kubernetes operation failed'): string {
  const kubernetesError = error as ErrorWithResponseBody;

  if (kubernetesError.response?.body?.message) {
    return kubernetesError.response.body.message;
  }

  return getErrorMessage(error, fallback);
}
