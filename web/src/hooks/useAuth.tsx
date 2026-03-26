import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { createContext, useContext, useState, type ReactNode } from "react";
import { api } from "../utils/api";

export interface User {
  id: string;
  username: string;
  role: "admin" | "viewer";
  usingDefaultPassword?: boolean;
}

interface AuthContextType {
  user: User | null;
  token: string | null;
  login: (username: string, password: string) => Promise<void>;
  changePassword: (currentPassword: string, newPassword: string) => Promise<void>;
  logout: () => void;
  isChangingPassword: boolean;
  isLoading: boolean;
}

const AuthContext = createContext<AuthContextType | null>(null);

export function useAuth() {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error("useAuth must be used within AuthProvider");
  }
  return context;
}

interface AuthProviderProps {
  children: ReactNode;
}

export function AuthProvider({ children }: AuthProviderProps) {
  const [token, setToken] = useState<string | null>(localStorage.getItem("token"));
  const queryClient = useQueryClient();

  const clearAuthState = () => {
    localStorage.removeItem("token");
    setToken(null);
    queryClient.clear();
  };

  // Get current user
  const { data: userData, isLoading } = useQuery({
    queryKey: ["me"],
    queryFn: async () => {
      if (!token) return null;
      try {
        return await api.get<{ user: User }>("/auth/me");
      } catch (error) {
        clearAuthState();
        return null;
      }
    },
    enabled: !!token,
    retry: false,
  });

  const loginMutation = useMutation({
    mutationFn: async ({ username, password }: { username: string; password: string }) => {
      return api.post<{ token: string; user: User }>("/auth/login", { username, password });
    },
    onSuccess: (data) => {
      localStorage.setItem("token", data.token);
      setToken(data.token);
      queryClient.setQueryData(["me"], { user: data.user });
    },
  });

  const changePasswordMutation = useMutation({
    mutationFn: async ({
      currentPassword,
      newPassword,
    }: {
      currentPassword: string;
      newPassword: string;
    }) => {
      await api.put("/auth/password", { currentPassword, newPassword });
    },
  });

  const logoutMutation = useMutation({
    mutationFn: async () => {
      if (token) {
        await api.post("/auth/logout");
      }
    },
    onSettled: () => {
      clearAuthState();
    },
  });

  const login = async (username: string, password: string) => {
    await loginMutation.mutateAsync({ username, password });
  };

  const changePassword = async (currentPassword: string, newPassword: string) => {
    await changePasswordMutation.mutateAsync({ currentPassword, newPassword });
  };

  const logout = () => {
    logoutMutation.mutate();
  };

  return (
    <AuthContext.Provider
      value={{
        user: userData?.user || null,
        token,
        login,
        changePassword,
        logout,
        isChangingPassword: changePasswordMutation.isPending,
        isLoading,
      }}
    >
      {children}
    </AuthContext.Provider>
  );
}
