import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { FileManager } from "@/components/system/file-manager";
import type { FileItem, FileStatisticsResp } from "@/types/system";

vi.mock("sonner", () => ({
  toast: {
    error: vi.fn(),
    info: vi.fn(),
    success: vi.fn()
  }
}));

vi.mock("@/hooks/use-permission", () => ({
  useHasAnyPermission: () => true,
  useHasEveryPermission: () => true
}));

const listFileMock = vi.fn();
const getFileStatisticsMock = vi.fn();

vi.mock("@/api/system/file", () => ({
  calcDirSize: vi.fn(),
  createDir: vi.fn(),
  deleteFile: vi.fn(),
  getFileStatistics: () => getFileStatisticsMock(),
  listFile: (query: unknown) => listFileMock(query),
  updateFile: vi.fn(),
  uploadFile: vi.fn()
}));

describe("FileManager", () => {
  beforeEach(() => {
    vi.spyOn(window, "open").mockImplementation(() => null);
    getFileStatisticsMock.mockResolvedValue(emptyStats());
    listFileMock.mockResolvedValue({
      list: [folderItem()],
      total: 1
    });
  });

  afterEach(() => {
    vi.restoreAllMocks();
    listFileMock.mockReset();
    getFileStatisticsMock.mockReset();
  });

  it("opens backend directories by parentPath instead of treating them as downloadable files", async () => {
    render(<FileManager />);

    const folder = await screen.findByRole("button", { name: "reports" });
    fireEvent.click(folder);

    await waitFor(() => {
      expect(screen.getByText("/reports")).toBeTruthy();
    });
    expect(window.open).not.toHaveBeenCalled();
    expect(listFileMock).toHaveBeenLastCalledWith(
      expect.objectContaining({
        parentPath: "/reports"
      })
    );
  });

  it("opens a rename dialog instead of a browser prompt", async () => {
    const promptSpy = vi.spyOn(window, "prompt").mockReturnValue("renamed");

    render(<FileManager />);

    fireEvent.click(await screen.findByRole("button", { name: "重命名" }));

    expect(promptSpy).not.toHaveBeenCalled();
    expect(screen.getByRole("dialog", { name: "重命名" })).toBeTruthy();
    expect(screen.getByDisplayValue("reports")).toBeTruthy();
  });
});

function folderItem(): FileItem {
  return {
    id: 11,
    name: "reports",
    originalName: "reports",
    size: 0,
    url: "/file/reports",
    parentPath: "/",
    path: "/reports",
    sha256: "",
    contentType: "",
    metadata: "{}",
    thumbnailSize: 0,
    thumbnailName: "",
    thumbnailMetadata: "",
    thumbnailUrl: "",
    extension: "",
    type: 0,
    storageId: 1,
    storageName: "本地存储",
    createUserString: "admin",
    createTime: "2026-05-29 20:00:00",
    updateUserString: "",
    updateTime: ""
  };
}

function emptyStats(): FileStatisticsResp {
  return {
    type: "",
    size: 0,
    number: 0,
    unit: "",
    data: []
  };
}
