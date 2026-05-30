import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { getImageCaptcha } from "@/api/auth";

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

describe("auth api wrappers", () => {
  const fetchMock = vi.fn<typeof fetch>(() =>
    okResponse({
      uuid: "captcha-uuid",
      img: "data:image/svg+xml;base64,abc",
      expireTime: 120,
      isEnabled: true
    })
  );

  beforeEach(() => {
    vi.stubGlobal("fetch", fetchMock);
  });

  afterEach(() => {
    vi.unstubAllGlobals();
    fetchMock.mockClear();
  });

  it("loads image captcha from the backend captcha endpoint", async () => {
    const captcha = await getImageCaptcha();

    expect(fetchMock.mock.calls[0]?.[0]).toBe("http://localhost:4398/captcha/image");
    expect(fetchMock.mock.calls[0]?.[1]).toMatchObject({ method: "GET" });
    expect(captcha.uuid).toBe("captcha-uuid");
    expect(captcha.img).toBe("data:image/svg+xml;base64,abc");
  });
});
