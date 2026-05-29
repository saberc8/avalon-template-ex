import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { listUser, deleteUser } from "@/api/system/user";
import { updateRolePermission } from "@/api/system/role";
import { clearMenuCache } from "@/api/system/menu";
import { listDept } from "@/api/system/dept";

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

describe("system api wrappers", () => {
  const fetchMock = vi.fn<typeof fetch>(() => okResponse());

  beforeEach(() => {
    vi.stubGlobal("fetch", fetchMock);
  });

  afterEach(() => {
    vi.unstubAllGlobals();
    fetchMock.mockClear();
  });

  it("uses the Vue-compatible user page and delete endpoints", async () => {
    await listUser({ page: 2, size: 20, description: "admin", sort: ["id,desc"] });
    await deleteUser(7);

    expect(fetchMock.mock.calls[0]?.[0]).toBe(
      "http://localhost:4398/system/user?page=2&size=20&description=admin&sort=id%2Cdesc"
    );
    expect(fetchMock.mock.calls[0]?.[1]).toMatchObject({ method: "GET" });
    expect(fetchMock.mock.calls[1]?.[0]).toBe("http://localhost:4398/system/user");
    expect(fetchMock.mock.calls[1]?.[1]).toMatchObject({
      method: "DELETE",
      body: JSON.stringify({ ids: [7] })
    });
  });

  it("uses exact role permission, menu cache, and department tree endpoints", async () => {
    await updateRolePermission(3, { menuIds: [1, 2], menuCheckStrictly: false });
    await clearMenuCache();
    await listDept({ status: 1 });

    expect(fetchMock.mock.calls[0]?.[0]).toBe("http://localhost:4398/system/role/3/permission");
    expect(fetchMock.mock.calls[0]?.[1]).toMatchObject({ method: "PUT" });
    expect(fetchMock.mock.calls[1]?.[0]).toBe("http://localhost:4398/system/menu/cache");
    expect(fetchMock.mock.calls[1]?.[1]).toMatchObject({ method: "DELETE" });
    expect(fetchMock.mock.calls[2]?.[0]).toBe("http://localhost:4398/system/dept/tree?status=1");
    expect(fetchMock.mock.calls[2]?.[1]).toMatchObject({ method: "GET" });
  });
});
