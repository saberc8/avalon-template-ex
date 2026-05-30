import { render, screen } from "@testing-library/react";
import { Pencil } from "lucide-react";
import { describe, expect, it } from "vitest";
import { TableActionButton, TableActions } from "@/components/table/table-actions";

describe("table actions", () => {
  it("renders compact action buttons with visible text labels", () => {
    render(
      <TableActions>
        <TableActionButton icon={Pencil} label="编辑" onClick={() => undefined} />
      </TableActions>
    );

    const button = screen.getByRole("button", { name: "编辑" });
    expect(button).toBeTruthy();
    expect(screen.getByText("编辑")).toBeTruthy();
  });
});
