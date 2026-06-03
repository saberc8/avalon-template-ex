import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { Dialog, DialogContent, DialogDescription, DialogTitle } from "@/components/ui/dialog";

describe("DialogContent", () => {
  it("caps tall dialog content to the viewport and allows vertical scrolling", () => {
    render(
      <Dialog open>
        <DialogContent>
          <DialogTitle>长表单</DialogTitle>
          <DialogDescription>验证长内容滚动</DialogDescription>
          <div style={{ height: 1200 }} />
        </DialogContent>
      </Dialog>
    );

    const dialog = screen.getByRole("dialog", { name: "长表单" });

    expect(dialog.className).toContain("max-h-[calc(100vh-2rem)]");
    expect(dialog.className).toContain("overflow-y-auto");
  });
});
