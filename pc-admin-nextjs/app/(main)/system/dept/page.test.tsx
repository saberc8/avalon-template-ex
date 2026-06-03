import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import DeptPage from "./page";
import { listDept } from "@/api/system/dept";
import type { DeptResp } from "@/types/system";

vi.mock("@/api/system/dept", () => ({
  addDept: vi.fn(),
  deleteDept: vi.fn(),
  exportDept: vi.fn(),
  getDept: vi.fn(),
  listDept: vi.fn(),
  updateDept: vi.fn()
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

const listDeptMock = vi.mocked(listDept);

function dept(overrides: Partial<DeptResp>): DeptResp {
  return {
    id: 1,
    name: "总部",
    sort: 1,
    status: 1,
    isSystem: false,
    description: "",
    createUserString: "admin",
    createTime: "2026-05-30T00:00:00Z",
    updateUserString: "admin",
    updateTime: "2026-05-30T00:00:00Z",
    parentId: 0,
    children: [],
    ...overrides
  };
}

describe("DeptPage tree table", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    listDeptMock.mockResolvedValue([
      dept({
        id: 1,
        name: "总部",
        children: [dept({ id: 2, name: "研发部", parentId: 1 })]
      })
    ]);
  });

  it("collapses and expands child department rows from the table row toggle", async () => {
    render(<DeptPage />);

    await screen.findByText("研发部");
    const collapse = screen.getByRole("button", { name: "收起 总部" });
    expect(collapse.getAttribute("aria-expanded")).toBe("true");

    fireEvent.click(collapse);

    await waitFor(() => expect(screen.queryByText("研发部")).toBeNull());
    const expand = screen.getByRole("button", { name: "展开 总部" });
    expect(expand.getAttribute("aria-expanded")).toBe("false");

    fireEvent.click(expand);

    expect(await screen.findByText("研发部")).toBeTruthy();
  });
});
