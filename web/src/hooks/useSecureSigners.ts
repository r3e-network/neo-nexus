import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "../utils/api";
import type {
  SecureSignerAttestationResult,
  SecureSignerCommandResult,
  SecureSignerOrchestrationPlan,
  CreateSecureSignerRequest,
  SecureSignerProfile,
  SecureSignerReadinessResult,
  SecureSignerTestResult,
  UpdateSecureSignerRequest,
} from "../../../src/types";

export function useSecureSigners() {
  return useQuery({
    queryKey: ["secure-signers"],
    queryFn: async () => {
      const response = await api.get<{ profiles: SecureSignerProfile[] }>("/secure-signers");
      return response.profiles;
    },
  });
}

export function useCreateSecureSigner() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (payload: CreateSecureSignerRequest) => {
      const response = await api.post<{ profile: SecureSignerProfile }>(
        "/secure-signers",
        payload as unknown as Record<string, unknown>,
      );
      return response.profile;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["secure-signers"] });
    },
  });
}

export function useUpdateSecureSigner() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async ({ id, payload }: { id: string; payload: UpdateSecureSignerRequest }) => {
      const response = await api.put<{ profile: SecureSignerProfile }>(
        `/secure-signers/${id}`,
        payload as unknown as Record<string, unknown>,
      );
      return response.profile;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["secure-signers"] });
    },
  });
}

export function useDeleteSecureSigner() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (id: string) => {
      await api.delete(`/secure-signers/${id}`);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["secure-signers"] });
    },
  });
}

export function useTestSecureSigner() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (id: string) => {
      const response = await api.post<{ result: SecureSignerTestResult }>(`/secure-signers/${id}/test`);
      return response.result;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["secure-signers"] });
    },
  });
}

export function useSecureSignerOrchestration(id?: string) {
  return useQuery({
    queryKey: ["secure-signers", id, "orchestration"],
    queryFn: async () => {
      if (!id) {
        throw new Error("Secure signer id is required");
      }

      const response = await api.get<{
        orchestration: SecureSignerOrchestrationPlan & { readiness: SecureSignerReadinessResult };
      }>(`/secure-signers/${id}/orchestration`);
      return response.orchestration;
    },
    enabled: !!id,
  });
}

export function useFetchSecureSignerAttestation() {
  return useMutation({
    mutationFn: async (id: string) => {
      const response = await api.post<{ attestation: SecureSignerAttestationResult }>(`/secure-signers/${id}/attestation`);
      return response.attestation;
    },
  });
}

export function useStartSecureSignerRecipient() {
  return useMutation({
    mutationFn: async ({ id, ciphertextBase64 }: { id: string; ciphertextBase64: string }) => {
      const response = await api.post<{ result: SecureSignerCommandResult }>(`/secure-signers/${id}/start-recipient`, {
        ciphertextBase64,
      });
      return response.result;
    },
  });
}
