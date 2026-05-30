import { render, screen } from "@testing-library/react";
import type { ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import DictPage from "./page";
import { listDict, listDictItem } from "@/api/system/dict";
import type { DictResp } from "@/types/system";

vi.mock("@/api/system/dict", () => ({
  addDict: vi.fn(),
  addDictItem: vi.fn(),
  clearDictCache: vi.fn(),
  deleteDict: vi.fn(),
  deleteDictItem: vi.fn(),
  getDict: vi.fn(),
  getDictItem: vi.fn(),
  listDict: vi.fn(),
  listDictItem: vi.fn(),
  updateDict: vi.fn(),
  updateDictItem: vi.fn()
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

const listDictMock = vi.mocked(listDict);
const listDictItemMock = vi.mocked(listDictItem);

function dict(overrides: Partial<DictResp>): DictResp {
  return {
    id: 1,
    name: "用户状态",
    code: "user_status",
    isSystem: false,
    description: "用户启用状态",
    createUserString: "admin",
    createTime: "2026-05-30T00:00:00Z",
    updateUserString: "admin",
    updateTime: "2026-05-30T00:00:00Z",
    ...overrides
  };
}

describe("DictPage layout", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    listDictMock.mockResolvedValue([
      dict({ id: 1 }),
      dict({ id: 2, name: "消息类型", code: "message_type", isSystem: true })
    ]);
    listDictItemMock.mockResolvedValue({ list: [], total: 0 });
  });

  it("keeps the left and right panels top aligned", async () => {
    render(<DictPage />);

    const layout = await screen.findByTestId("dict-layout");
    expect(layout.className).toContain("items-start");
    expect(screen.getByTestId("dict-list-panel").className).toContain("self-start");
    expect(screen.getByTestId("dict-items-panel").className).toContain("self-start");
    expect(screen.getByTestId("dict-items-panel").className).toContain("content-start");
  });

  it("moves dict card actions into a top-right hover/focus overlay", async () => {
    render(<DictPage />);

    const card = await screen.findByTestId("dict-card-1");
    expect(card.className).toContain("group");
    expect(card.className).toContain("relative");

    const actions = screen.getByTestId("dict-card-actions-1");
    expect(actions.className).toContain("absolute");
    expect(actions.className).toContain("right-2");
    expect(actions.className).toContain("top-2");
    expect(actions.className).toContain("opacity-0");
    expect(actions.className).toContain("group-hover:opacity-100");
    expect(actions.className).toContain("group-focus-within:opacity-100");

    expect(screen.getByRole("button", { name: "编辑 用户状态" }).textContent).not.toContain("编辑");
    expect(screen.getByRole("button", { name: "删除 用户状态" }).textContent).not.toContain("删除");
  });
});
