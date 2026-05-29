"use client";

import { useCurrentUser } from "@/hooks/use-current-user";

export function useHasAnyPermission(permissions: string[]) {
  return useCurrentUser().hasAnyPermission(permissions);
}

export function useHasEveryPermission(permissions: string[]) {
  return useCurrentUser().hasEveryPermission(permissions);
}
