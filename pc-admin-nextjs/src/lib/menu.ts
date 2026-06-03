import type { RouteItem } from "@/types/auth";

const BUTTON_TYPE = 3;
const ENABLED_STATUS = 1;
const ALWAYS_ACCESSIBLE_PATHS = new Set(["/dashboard/workplace", "/user/profile"]);

export function visibleMenuRoutes(routes: RouteItem[]) {
  return sortRoutes(routes).filter(isVisibleMenuRoute);
}

export function flattenVisibleRoutes(routes: RouteItem[]) {
  const result: RouteItem[] = [];

  for (const route of visibleMenuRoutes(routes)) {
    result.push(route);
    result.push(...flattenVisibleRoutes(route.children));
  }

  return result;
}

export function firstAccessiblePath(routes: RouteItem[]) {
  return findFirstAccessiblePath(routes) || "/dashboard/workplace";
}

function findFirstAccessiblePath(routes: RouteItem[]): string {
  for (const route of visibleMenuRoutes(routes)) {
    const childPath = findFirstAccessiblePath(route.children);
    if (childPath) {
      return childPath;
    }
    const href = routeHref(route);
    if (href) {
      return href;
    }
  }

  return "";
}

export function findRouteTrail(routes: RouteItem[], pathname: string): RouteItem[] {
  const normalizedPathname = stripQuery(pathname);

  for (const route of visibleMenuRoutes(routes)) {
    const href = routeHref(route);
    if (href && stripQuery(href) === normalizedPathname) {
      return [route];
    }

    const childTrail = findRouteTrail(route.children, pathname);
    if (childTrail.length > 0) {
      return [route, ...childTrail];
    }
  }

  return [];
}

export function isPathAccessible(routes: RouteItem[], pathname: string) {
  const currentPath = stripQuery(pathname);
  if (ALWAYS_ACCESSIBLE_PATHS.has(currentPath)) {
    return true;
  }

  return routeLeaves(routes).some((route) => routeIsActive(route, currentPath));
}

export function routeHref(route: RouteItem) {
  return route.path || route.redirect || "";
}

export function routeIsActive(route: RouteItem, pathname: string) {
  const href = routeHref(route);
  if (!href) {
    return false;
  }

  const routePath = stripQuery(href);
  const currentPath = stripQuery(pathname);
  return currentPath === routePath || currentPath.startsWith(`${routePath}/`);
}

function isVisibleMenuRoute(route: RouteItem) {
  return route.status === ENABLED_STATUS && !route.isHidden && route.type !== BUTTON_TYPE;
}

function sortRoutes(routes: RouteItem[]) {
  return [...routes].sort((left, right) => left.sort - right.sort || left.id - right.id);
}

function stripQuery(path: string) {
  return path.split("?")[0] || path;
}

function routeLeaves(routes: RouteItem[]) {
  const leaves: RouteItem[] = [];

  for (const route of visibleMenuRoutes(routes)) {
    const children = visibleMenuRoutes(route.children);
    if (children.length === 0) {
      leaves.push(route);
    } else {
      leaves.push(...routeLeaves(children));
    }
  }

  return leaves;
}
