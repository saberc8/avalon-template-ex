import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { listDictItem, clearDictCache } from "@/api/system/dict";
import { createDir, listFile } from "@/api/system/file";
import { listOption, resetOptionValue } from "@/api/system/option";
import { setDefaultStorage } from "@/api/system/storage";
import { listClient } from "@/api/system/client";

function okResponse(data: unknown = true) {
  return Promise.resolve(
    new Response(
      JSON.stringify({
        code: "200",
        data,
        msg: "成功",
        success: true,
        timestamp: "1"
      }),
      {
        status: 200,
        headers: { "Content-Type": "application/json" }
      }
    )
  );
}

describe("system api wrappers task 13", () => {
  const fetchMock = vi.fn<typeof fetch>(() => okResponse());

  beforeEach(() => {
    vi.stubGlobal("fetch", fetchMock);
  });

  afterEach(() => {
    vi.unstubAllGlobals();
    fetchMock.mockClear();
  });

  it("uses dict item and dict cache endpoints", async () => {
    await listDictItem({ page: 1, size: 10, dictId: 100, status: 1 });
    await clearDictCache("user_status");

    expect(fetchMock.mock.calls[0]?.[0]).toBe(
      "http://localhost:4398/system/dict/item?page=1&size=10&dictId=100&status=1"
    );
    expect(fetchMock.mock.calls[0]?.[1]).toMatchObject({ method: "GET" });
    expect(fetchMock.mock.calls[1]?.[0]).toBe("http://localhost:4398/system/dict/cache/user_status");
    expect(fetchMock.mock.calls[1]?.[1]).toMatchObject({ method: "DELETE" });
  });

  it("uses file list and directory endpoints", async () => {
    await listFile({ page: 1, size: 20, parentPath: "/docs", sort: ["id,desc"] });
    await createDir("/docs", "reports");

    expect(fetchMock.mock.calls[0]?.[0]).toBe(
      "http://localhost:4398/system/file?page=1&size=20&parentPath=%2Fdocs&sort=id%2Cdesc"
    );
    expect(fetchMock.mock.calls[1]?.[0]).toBe("http://localhost:4398/system/file/dir");
    expect(fetchMock.mock.calls[1]?.[1]).toMatchObject({
      method: "POST",
      body: JSON.stringify({ parentPath: "/docs", originalName: "reports" })
    });
  });

  it("uses option, storage, and client endpoints", async () => {
    await listOption({ category: "SITE" });
    await resetOptionValue({ category: "LOGIN" });
    await setDefaultStorage(5);
    await listClient({ page: 2, size: 10, clientType: "PC", authType: ["ACCOUNT"], status: 1 });

    expect(fetchMock.mock.calls[0]?.[0]).toBe("http://localhost:4398/system/option?category=SITE");
    expect(fetchMock.mock.calls[1]?.[0]).toBe("http://localhost:4398/system/option/value");
    expect(fetchMock.mock.calls[1]?.[1]).toMatchObject({ method: "PATCH" });
    expect(fetchMock.mock.calls[2]?.[0]).toBe("http://localhost:4398/system/storage/5/default");
    expect(fetchMock.mock.calls[2]?.[1]).toMatchObject({ method: "PUT" });
    expect(fetchMock.mock.calls[3]?.[0]).toBe(
      "http://localhost:4398/system/client?page=2&size=10&clientType=PC&authType=ACCOUNT&status=1"
    );
  });
});
