import { describe, expect, it } from "vitest";
import { hasAnyPermission, hasEveryPermission, hasRole } from "@/lib/permission";

describe("permission helpers", () => {
  it("allows admin and wildcard permissions to access protected actions", () => {
    expect(hasAnyPermission(["system:user:delete"], [], ["admin"])).toBe(true);
    expect(hasAnyPermission(["system:user:delete"], ["*:*:*"], [])).toBe(true);
    expect(hasAnyPermission(["system:user:delete"], ["*"], [])).toBe(true);
  });

  it("checks any and every required permission for normal users", () => {
    const userPermissions = ["system:user:list", "system:user:create"];

    expect(hasAnyPermission(["system:user:update", "system:user:list"], userPermissions, [])).toBe(true);
    expect(hasAnyPermission(["system:user:delete"], userPermissions, [])).toBe(false);
    expect(hasEveryPermission(["system:user:list", "system:user:create"], userPermissions, [])).toBe(true);
    expect(hasEveryPermission(["system:user:list", "system:user:delete"], userPermissions, [])).toBe(false);
  });

  it("treats empty requirements as visible by default", () => {
    expect(hasAnyPermission([], [], [])).toBe(true);
    expect(hasEveryPermission([], [], [])).toBe(true);
  });

  it("checks roles without granting non-admin roles all permissions", () => {
    expect(hasRole("general", ["general"])).toBe(true);
    expect(hasRole("admin", ["general"])).toBe(false);
    expect(hasAnyPermission(["system:role:delete"], [], ["general"])).toBe(false);
  });
});
