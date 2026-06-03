import { render, screen } from "@testing-library/react";
import type { ReactNode } from "react";
import { describe, expect, it, vi } from "vitest";
import ConfigPage from "./page";

vi.mock("@/components/permission/permission-gate", () => ({
  PermissionGate: ({ children }: { children: ReactNode }) => <>{children}</>
}));

vi.mock("@/components/system/config-client", () => ({
  ConfigClient: () => <div>客户端配置内容</div>
}));

vi.mock("@/components/system/config-login", () => ({
  ConfigLogin: () => <div>登录配置内容</div>
}));

vi.mock("@/components/system/config-security", () => ({
  ConfigSecurity: () => <div>安全配置内容</div>
}));

vi.mock("@/components/system/config-site", () => ({
  ConfigSite: () => <div>网站配置内容</div>
}));

vi.mock("@/components/system/config-storage", () => ({
  ConfigStorage: () => <div>存储配置内容</div>
}));

describe("ConfigPage layout", () => {
  it("uses a left card navigation and top-aligned content panel", () => {
    render(<ConfigPage />);

    expect(screen.getByTestId("config-layout").className).toContain("items-start");
    expect(screen.getByTestId("config-list-panel").className).toContain("self-start");
    expect(screen.getByTestId("config-content-panel").className).toContain("self-start");
    expect(screen.getByTestId("config-content-panel").className).toContain("content-start");
  });

  it("renders config sections as hoverable cards with top-right action affordances", () => {
    render(<ConfigPage />);

    const card = screen.getByTestId("config-card-site");
    expect(card.className).toContain("group");
    expect(card.className).toContain("relative");
    expect(card.className).toContain("pr-16");

    const action = screen.getByTestId("config-card-action-site");
    expect(action.className).toContain("absolute");
    expect(action.className).toContain("right-2");
    expect(action.className).toContain("top-2");
    expect(action.className).toContain("opacity-0");
    expect(action.className).toContain("group-hover:opacity-100");
    expect(action.className).toContain("group-focus-visible:opacity-100");
  });
});
