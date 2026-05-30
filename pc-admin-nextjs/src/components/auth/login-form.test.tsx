import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { LoginForm } from "@/components/auth/login-form";
import {
  accountLogin,
  getImageCaptcha,
  getUserInfo,
  getUserRoutes
} from "@/api/auth";

vi.mock("@/api/auth", () => ({
  accountLogin: vi.fn(),
  getImageCaptcha: vi.fn(),
  getUserInfo: vi.fn(),
  getUserRoutes: vi.fn()
}));

const replaceMock = vi.fn();
const routerMock = { replace: replaceMock };

vi.mock("next/navigation", () => ({
  useRouter: () => routerMock
}));

vi.mock("sonner", () => ({
  toast: {
    error: vi.fn(),
    success: vi.fn()
  }
}));

vi.mock("@/lib/menu", () => ({
  firstAccessiblePath: () => "/dashboard/workplace"
}));

const getImageCaptchaMock = vi.mocked(getImageCaptcha);
const accountLoginMock = vi.mocked(accountLogin);
const getUserInfoMock = vi.mocked(getUserInfo);
const getUserRoutesMock = vi.mocked(getUserRoutes);

describe("LoginForm captcha", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    window.localStorage.clear();
    getImageCaptchaMock.mockResolvedValue({
      uuid: "captcha-uuid-1",
      img: "data:image/svg+xml;base64,abc",
      expireTime: 120,
      isEnabled: true
    });
    accountLoginMock.mockResolvedValue({ token: "token-1", expire: "2099-01-01T00:00:00Z" });
    getUserInfoMock.mockResolvedValue({} as Awaited<ReturnType<typeof getUserInfo>>);
    getUserRoutesMock.mockResolvedValue([]);
  });

  it("loads and renders the image captcha on mount", async () => {
    render(<LoginForm />);

    expect(getImageCaptchaMock).toHaveBeenCalledTimes(1);
    const image = await screen.findByRole("img", { name: "验证码" });
    expect(image.getAttribute("src")).toBe("data:image/svg+xml;base64,abc");
  });

  it("submits the captcha API uuid instead of a hardcoded local uuid", async () => {
    render(<LoginForm />);

    await screen.findByRole("img", { name: "验证码" });
    fireEvent.change(screen.getByLabelText("验证码"), { target: { value: "1234" } });
    fireEvent.click(screen.getByRole("button", { name: "登录" }));

    await waitFor(() => expect(accountLoginMock).toHaveBeenCalledTimes(1));
    expect(accountLoginMock).toHaveBeenCalledWith(
      expect.objectContaining({
        username: "admin",
        password: "admin123",
        captcha: "1234",
        uuid: "captcha-uuid-1"
      })
    );
  });

  it("hides captcha controls and omits uuid when the backend disables image captcha", async () => {
    getImageCaptchaMock.mockResolvedValueOnce({
      uuid: "",
      img: "",
      expireTime: 120,
      isEnabled: false
    });

    render(<LoginForm />);

    await waitFor(() => expect(getImageCaptchaMock).toHaveBeenCalledTimes(1));
    expect(screen.queryByRole("img", { name: "验证码" })).toBeNull();
    expect(screen.queryByLabelText("验证码")).toBeNull();

    fireEvent.click(screen.getByRole("button", { name: "登录" }));

    await waitFor(() => expect(accountLoginMock).toHaveBeenCalledTimes(1));
    expect(accountLoginMock).toHaveBeenCalledWith(
      expect.objectContaining({
        captcha: undefined,
        uuid: undefined
      })
    );
  });

  it("refreshes the captcha after a failed login attempt", async () => {
    getImageCaptchaMock
      .mockResolvedValueOnce({
        uuid: "captcha-uuid-1",
        img: "data:image/svg+xml;base64,abc",
        expireTime: 120,
        isEnabled: true
      })
      .mockResolvedValueOnce({
        uuid: "captcha-uuid-2",
        img: "data:image/svg+xml;base64,def",
        expireTime: 120,
        isEnabled: true
      });
    accountLoginMock.mockRejectedValueOnce(new Error("登录失败"));

    render(<LoginForm />);

    await screen.findByRole("img", { name: "验证码" });
    fireEvent.change(screen.getByLabelText("验证码"), { target: { value: "1234" } });
    fireEvent.click(screen.getByRole("button", { name: "登录" }));

    await waitFor(() => expect(getImageCaptchaMock).toHaveBeenCalledTimes(2));
    expect(screen.getByRole("img", { name: "验证码" }).getAttribute("src")).toBe(
      "data:image/svg+xml;base64,def"
    );
  });

  it("handles captcha loading failures without leaving an unhandled rejection", async () => {
    getImageCaptchaMock.mockRejectedValueOnce(new Error("验证码加载失败"));

    render(<LoginForm />);

    await waitFor(() => expect(getImageCaptchaMock).toHaveBeenCalledTimes(1));
  });
});
