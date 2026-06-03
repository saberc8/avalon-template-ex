import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { deleteJob, listJob, listJobLog, runJob, updateJobStatus } from "@/api/schedule/job";

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

describe("schedule job api wrappers", () => {
  const fetchMock = vi.fn<typeof fetch>(() => okResponse());

  beforeEach(() => {
    vi.stubGlobal("fetch", fetchMock);
  });

  afterEach(() => {
    vi.unstubAllGlobals();
    fetchMock.mockClear();
  });

  it("uses schedule job page, status, run, delete, and log endpoints", async () => {
    await listJob({ page: 2, size: 20, description: "sync", sort: ["id,desc"] });
    await updateJobStatus(7, 1);
    await runJob(7);
    await deleteJob(7);
    await listJobLog(7, { page: 1, size: 10 });

    expect(fetchMock.mock.calls[0]?.[0]).toBe(
      "http://localhost:4398/schedule/job/page?page=2&size=20&description=sync&sort=id%2Cdesc"
    );
    expect(fetchMock.mock.calls[1]?.[0]).toBe("http://localhost:4398/schedule/job/7/status");
    expect(fetchMock.mock.calls[1]?.[1]).toMatchObject({ method: "PATCH" });
    expect(fetchMock.mock.calls[2]?.[0]).toBe("http://localhost:4398/schedule/job/7/run");
    expect(fetchMock.mock.calls[2]?.[1]).toMatchObject({ method: "POST" });
    expect(fetchMock.mock.calls[3]?.[0]).toBe("http://localhost:4398/schedule/job");
    expect(fetchMock.mock.calls[3]?.[1]).toMatchObject({
      method: "DELETE",
      body: JSON.stringify({ ids: [7] })
    });
    expect(fetchMock.mock.calls[4]?.[0]).toBe("http://localhost:4398/schedule/job/7/log?page=1&size=10");
  });
});
