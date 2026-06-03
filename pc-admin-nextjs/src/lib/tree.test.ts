import { describe, expect, it } from "vitest";
import { flattenTree, flattenVisibleTree, mapTree } from "@/lib/tree";

interface Node {
  id: number;
  title: string;
  children: Node[];
}

const tree: Node[] = [
  {
    id: 1,
    title: "系统管理",
    children: [
      { id: 2, title: "用户管理", children: [] },
      {
        id: 3,
        title: "角色管理",
        children: [{ id: 4, title: "权限配置", children: [] }]
      }
    ]
  }
];

describe("tree helpers", () => {
  it("flattens tree nodes with depth and parent references", () => {
    expect(flattenTree(tree).map((item) => [item.node.id, item.depth, item.parent?.id ?? 0])).toEqual([
      [1, 0, 0],
      [2, 1, 1],
      [3, 1, 1],
      [4, 2, 3]
    ]);
  });

  it("maps every node while preserving children", () => {
    expect(mapTree(tree, (node) => ({ label: node.title, children: node.children }))).toEqual([
      {
        label: "系统管理",
        children: [
          { label: "用户管理", children: [] },
          {
            label: "角色管理",
            children: [{ label: "权限配置", children: [] }]
          }
        ]
      }
    ]);
  });

  it("flattens only visible tree nodes when parents are collapsed", () => {
    expect(
      flattenVisibleTree(tree, new Set([3])).map((item) => [
        item.node.id,
        item.depth,
        item.hasChildren,
        item.expanded
      ])
    ).toEqual([
      [1, 0, true, true],
      [2, 1, false, false],
      [3, 1, true, false]
    ]);
  });

  it("hides all descendants when a root node is collapsed", () => {
    expect(flattenVisibleTree(tree, new Set([1])).map((item) => item.node.id)).toEqual([1]);
  });
});
