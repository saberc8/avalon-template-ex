"use client";

import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode
} from "react";
import { useRouter } from "next/navigation";
import {
  getUserInfo,
  getUserRoutes,
  logout as requestLogout
} from "@/api/auth";
import { clearToken, getToken } from "@/lib/auth";
import { firstAccessiblePath } from "@/lib/menu";
import { hasAnyPermission, hasEveryPermission } from "@/lib/permission";
import type { RouteItem, UserInfo } from "@/types/auth";

interface CurrentUserContextValue {
  user: UserInfo | null;
  routes: RouteItem[];
  loading: boolean;
  error: string;
  landingPath: string;
  reload: () => Promise<void>;
  logout: () => Promise<void>;
  hasAnyPermission: (permissions: string[]) => boolean;
  hasEveryPermission: (permissions: string[]) => boolean;
}

const CurrentUserContext = createContext<CurrentUserContextValue | null>(null);

export function CurrentUserProvider({ children }: { children: ReactNode }) {
  const router = useRouter();
  const [user, setUser] = useState<UserInfo | null>(null);
  const [routes, setRoutes] = useState<RouteItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");

  const reload = useCallback(async () => {
    const token = getToken();
    if (!token) {
      setUser(null);
      setRoutes([]);
      setLoading(false);
      router.replace("/login");
      return;
    }

    setLoading(true);
    setError("");

    try {
      const [nextUser, nextRoutes] = await Promise.all([getUserInfo(), getUserRoutes()]);
      setUser(nextUser);
      setRoutes(nextRoutes);
    } catch (currentError) {
      clearToken();
      setUser(null);
      setRoutes([]);
      setError(currentError instanceof Error ? currentError.message : "用户状态已失效");
      router.replace("/login");
    } finally {
      setLoading(false);
    }
  }, [router]);

  useEffect(() => {
    void reload();
  }, [reload]);

  const logout = useCallback(async () => {
    try {
      await requestLogout();
    } finally {
      clearToken();
      setUser(null);
      setRoutes([]);
      router.replace("/login");
    }
  }, [router]);

  const permissions = user?.permissions ?? [];
  const roles = user?.roles ?? [];

  const value = useMemo<CurrentUserContextValue>(
    () => ({
      user,
      routes,
      loading,
      error,
      landingPath: firstAccessiblePath(routes),
      reload,
      logout,
      hasAnyPermission: (requiredPermissions) =>
        hasAnyPermission(requiredPermissions, permissions, roles),
      hasEveryPermission: (requiredPermissions) =>
        hasEveryPermission(requiredPermissions, permissions, roles)
    }),
    [error, loading, logout, permissions, reload, roles, routes, user]
  );

  return <CurrentUserContext.Provider value={value}>{children}</CurrentUserContext.Provider>;
}

export function useCurrentUser() {
  const context = useContext(CurrentUserContext);
  if (!context) {
    throw new Error("useCurrentUser must be used inside CurrentUserProvider");
  }
  return context;
}
