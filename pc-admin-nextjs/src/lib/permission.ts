const ADMIN_ROLE_CODE = "admin";
const WILDCARD_PERMISSIONS = new Set(["*", "*:*:*"]);

export function hasRole(role: string, roles: string[]) {
  return roles.includes(role);
}

export function hasAnyRole(requiredRoles: string[], roles: string[]) {
  if (requiredRoles.length === 0) {
    return true;
  }
  return requiredRoles.some((role) => hasRole(role, roles));
}

export function hasAnyPermission(
  requiredPermissions: string[],
  userPermissions: string[],
  roles: string[]
) {
  if (requiredPermissions.length === 0) {
    return true;
  }
  if (isAdmin(roles) || hasWildcardPermission(userPermissions)) {
    return true;
  }
  return requiredPermissions.some((permission) => userPermissions.includes(permission));
}

export function hasEveryPermission(
  requiredPermissions: string[],
  userPermissions: string[],
  roles: string[]
) {
  if (requiredPermissions.length === 0) {
    return true;
  }
  if (isAdmin(roles) || hasWildcardPermission(userPermissions)) {
    return true;
  }
  return requiredPermissions.every((permission) => userPermissions.includes(permission));
}

function isAdmin(roles: string[]) {
  return roles.includes(ADMIN_ROLE_CODE);
}

function hasWildcardPermission(userPermissions: string[]) {
  return userPermissions.some((permission) => WILDCARD_PERMISSIONS.has(permission));
}
