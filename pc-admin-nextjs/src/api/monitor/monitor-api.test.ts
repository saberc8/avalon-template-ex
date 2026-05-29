import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { listOnlineUser, kickout } from "@/api/monitor/online";
import { exportOperationLog, getLog, listLog } from "@/api/monitor/log";
import { updateUserBaseInfo, updateUserPassword } from "@/api/system/user-profile";

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

describe("monitor and profile api wrappers", () => {
  const fetchMock = vi.fn<typeof fetch>(() => okResponse());

  beforeEach(() => {
    vi.stubGlobal("fetch", fetchMock);
  });

  afterEach(() => {
    vi.unstubAllGlobals();
    fetchMock.mockClear();
  });

  it("uses online user endpoints", async () => {
    await listOnlineUser({ page: 1, size: 20, nickname: "admin", sort: ["loginTime,desc"] });
    await kickout("token-1");

    expect(fetchMock.mock.calls[0]?.[0]).toBe(
      "http://localhost:4398/monitor/online?page=1&size=20&nickname=admin&sort=loginTime%2Cdesc"
    );
    expect(fetchMock.mock.calls[1]?.[0]).toBe("http://localhost:4398/monitor/online/token-1");
    expect(fetchMock.mock.calls[1]?.[1]).toMatchObject({ method: "DELETE" });
  });

  it("uses log list, detail, and export endpoints", async () => {
    await listLog({ page: 2, size: 10, module: "登录", status: 1 });
    await getLog(9);
    await exportOperationLog({ module: "系统" });

    expect(fetchMock.mock.calls[0]?.[0]).toBe(
      "http://localhost:4398/system/log?page=2&size=10&module=%E7%99%BB%E5%BD%95&status=1"
    );
    expect(fetchMock.mock.calls[1]?.[0]).toBe("http://localhost:4398/system/log/9");
    expect(fetchMock.mock.calls[2]?.[0]).toBe("http://localhost:4398/system/log/export/operation?module=%E7%B3%BB%E7%BB%9F");
  });

  it("uses profile update endpoints", async () => {
    await updateUserBaseInfo({ nickname: "管理员", gender: 1 });
    await updateUserPassword({ oldPassword: "old", newPassword: "new" });

    expect(fetchMock.mock.calls[0]?.[0]).toBe("http://localhost:4398/user/profile/basic/info");
    expect(fetchMock.mock.calls[0]?.[1]).toMatchObject({ method: "PATCH" });
    expect(fetchMock.mock.calls[1]?.[0]).toBe("http://localhost:4398/user/profile/password");
    expect(fetchMock.mock.calls[1]?.[1]).toMatchObject({
      method: "PATCH",
      body: JSON.stringify({ oldPassword: "old", newPassword: "new" })
    });
  });
});
