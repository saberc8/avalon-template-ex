"use client";

import type { ReactNode } from "react";
import { useHasAnyPermission } from "@/hooks/use-permission";

interface PermissionGateProps {
  permissions: string[];
  children: ReactNode;
}

export function PermissionGate({ permissions, children }: PermissionGateProps) {
  const allowed = useHasAnyPermission(permissions);
  if (!allowed) {
    return null;
  }
  return <>{children}</>;
}
