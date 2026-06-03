import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import MenuPage from "./page";
import { listMenu } from "@/api/system/menu";
import type { MenuResp } from "@/types/system";

vi.mock("@/api/system/menu", () => ({
  addMenu: vi.fn(),
  clearMenuCache: vi.fn(),
  deleteMenu: vi.fn(),
  getMenu: vi.fn(),
  listMenu: vi.fn(),
  updateMenu: vi.fn()
}));

vi.mock("@/components/permission/permission-gate", () => ({
  PermissionGate: ({ children }: { children: ReactNode }) => <>{children}</>
}));

vi.mock("sonner", () => ({
  toast: {
    error: vi.fn(),
    success: vi.fn()
  }
}));

const listMenuMock = vi.mocked(listMenu);

function menu(overrides: Partial<MenuResp>): MenuResp {
  return {
    id: 1,
    title: "系统管理",
    parentId: 0,
    type: 1,
    path: "/system",
    name: "System",
    component: "",
    redirect: "",
    icon: "settings",
    isExternal: false,
    isCache: true,
    isHidden: false,
    permission: "",
    sort: 1,
    status: 1,
    createUserString: "admin",
    createTime: "2026-05-30T00:00:00Z",
    updateUserString: "admin",
    updateTime: "2026-05-30T00:00:00Z",
    children: [],
    ...overrides
  };
}

describe("MenuPage tree table", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    listMenuMock.mockResolvedValue([
      menu({
        id: 1,
        title: "系统管理",
        children: [menu({ id: 2, title: "用户管理", parentId: 1, type: 2, path: "/system/user" })]
      })
    ]);
  });

  it("collapses and expands child menu rows from the table row toggle", async () => {
    render(<MenuPage />);

    await screen.findByText("用户管理");
    const collapse = screen.getByRole("button", { name: "收起 系统管理" });
    expect(collapse.getAttribute("aria-expanded")).toBe("true");

    fireEvent.click(collapse);

    await waitFor(() => expect(screen.queryByText("用户管理")).toBeNull());
    const expand = screen.getByRole("button", { name: "展开 系统管理" });
    expect(expand.getAttribute("aria-expanded")).toBe("false");

    fireEvent.click(expand);

    expect(await screen.findByText("用户管理")).toBeTruthy();
  });
});
