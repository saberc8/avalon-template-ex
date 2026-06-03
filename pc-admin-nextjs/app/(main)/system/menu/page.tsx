"use client";

import { ChevronDown, ChevronRight, Eraser, FilePlus2, Pencil, RefreshCw, Trash2 } from "lucide-react";
import { useCallback, useEffect, useMemo, useState } from "react";
import type { ColumnDef } from "@tanstack/react-table";
import { toast } from "sonner";
import { addMenu, clearMenuCache, deleteMenu, getMenu, listMenu, updateMenu } from "@/api/system/menu";
import { DataTable } from "@/components/table/data-table";
import { TableActionButton, TableActions } from "@/components/table/table-actions";
import { PermissionGate } from "@/components/permission/permission-gate";
import { MenuForm } from "@/components/system/menu-form";
import { StatusBadge } from "@/components/system/status-badge";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue
} from "@/components/ui/select";
import { flattenVisibleTree } from "@/lib/tree";
import type { MenuCommand, MenuResp } from "@/types/system";

type MenuRow = MenuResp & { depth: number; hasChildren: boolean; expanded: boolean };

const typeLabels: Record<number, string> = {
  1: "目录",
  2: "菜单",
  3: "按钮"
};

export default function MenuPage() {
  const [menus, setMenus] = useState<MenuResp[]>([]);
  const [title, setTitle] = useState("");
  const [status, setStatus] = useState("all");
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [editorOpen, setEditorOpen] = useState(false);
  const [editingMenu, setEditingMenu] = useState<MenuResp | null>(null);
  const [collapsedMenuIds, setCollapsedMenuIds] = useState<Set<number>>(new Set());

  const loadMenus = useCallback(async () => {
    setLoading(true);
    try {
      setMenus(
        await listMenu({
          title: title || undefined,
          status: status === "all" ? undefined : Number(status)
        })
      );
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "菜单列表加载失败");
    } finally {
      setLoading(false);
    }
  }, [status, title]);

  useEffect(() => {
    void loadMenus();
  }, [loadMenus]);

  const rows = useMemo<MenuRow[]>(
    () =>
      flattenVisibleTree(menus, collapsedMenuIds).map(({ node, depth, hasChildren, expanded }) => ({
        ...node,
        depth,
        hasChildren,
        expanded
      })),
    [collapsedMenuIds, menus]
  );

  const columns = useMemo<ColumnDef<MenuRow>[]>(
    () => [
      {
        header: "菜单名称",
        cell: ({ row }) => (
          <div className="flex items-center gap-2" style={{ paddingLeft: row.original.depth * 20 }}>
            {row.original.hasChildren ? (
              <Button
                type="button"
                size="icon"
                variant="ghost"
                aria-expanded={row.original.expanded}
                aria-label={`${row.original.expanded ? "收起" : "展开"} ${row.original.title}`}
                className="size-7 shrink-0"
                onClick={() => toggleMenu(row.original.id)}
              >
                {row.original.expanded ? <ChevronDown /> : <ChevronRight />}
              </Button>
            ) : (
              <span className="size-7 shrink-0" aria-hidden="true" />
            )}
            <span>{row.original.title}</span>
          </div>
        )
      },
      {
        header: "类型",
        cell: ({ row }) => typeLabels[row.original.type]
      },
      { accessorKey: "path", header: "路由" },
      { accessorKey: "permission", header: "权限标识" },
      {
        header: "状态",
        cell: ({ row }) => <StatusBadge status={row.original.status} />
      },
      { accessorKey: "sort", header: "排序" },
      {
        id: "actions",
        header: "操作",
        cell: ({ row }) => (
          <TableActions>
            <PermissionGate permissions={["system:menu:update"]}>
              <TableActionButton icon={Pencil} label="编辑" onClick={() => void openEditor(row.original.id)} />
            </PermissionGate>
            <PermissionGate permissions={["system:menu:delete"]}>
              <TableActionButton icon={Trash2} label="删除" destructive onClick={() => void removeMenu(row.original.id)} />
            </PermissionGate>
          </TableActions>
        )
      }
    ],
    []
  );

  function toggleMenu(id: number) {
    setCollapsedMenuIds((current) => {
      const next = new Set(current);
      if (next.has(id)) {
        next.delete(id);
      } else {
        next.add(id);
      }
      return next;
    });
  }

  async function openEditor(id?: number) {
    if (!id) {
      setEditingMenu(null);
      setEditorOpen(true);
      return;
    }
    try {
      setEditingMenu(await getMenu(id));
      setEditorOpen(true);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "菜单详情加载失败");
    }
  }

  async function saveMenu(command: MenuCommand) {
    setSaving(true);
    try {
      if (editingMenu) {
        await updateMenu(editingMenu.id, command);
      } else {
        await addMenu(command);
      }
      setEditorOpen(false);
      await loadMenus();
      toast.success("菜单已保存");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "菜单保存失败");
    } finally {
      setSaving(false);
    }
  }

  async function removeMenu(id: number) {
    if (!window.confirm("确认删除该菜单？")) {
      return;
    }
    try {
      await deleteMenu(id);
      await loadMenus();
      toast.success("菜单已删除");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "菜单删除失败");
    }
  }

  async function clearCache() {
    try {
      await clearMenuCache();
      toast.success("菜单缓存已清理");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "菜单缓存清理失败");
    }
  }

  return (
    <div className="mx-auto flex w-full max-w-7xl flex-col gap-4">
      <section className="flex flex-col gap-3 rounded-lg border bg-background p-4 md:flex-row md:items-end md:justify-between">
        <div className="grid gap-3 md:grid-cols-2 md:items-end">
          <div className="grid gap-2 md:w-72">
            <span className="text-sm font-medium">菜单名称</span>
            <Input value={title} onChange={(event) => setTitle(event.target.value)} />
          </div>
          <div className="grid gap-2 md:w-44">
            <span className="text-sm font-medium">状态</span>
            <Select value={status} onValueChange={setStatus}>
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">全部</SelectItem>
                <SelectItem value="1">启用</SelectItem>
                <SelectItem value="2">禁用</SelectItem>
              </SelectContent>
            </Select>
          </div>
        </div>
        <div className="flex flex-wrap gap-2">
          <Button variant="outline" onClick={() => void loadMenus()}>
            <RefreshCw />
            刷新
          </Button>
          <PermissionGate permissions={["system:menu:create"]}>
            <Button onClick={() => void openEditor()}>
              <FilePlus2 />
              新增
            </Button>
          </PermissionGate>
          <PermissionGate permissions={["system:menu:clearCache"]}>
            <Button variant="outline" onClick={() => void clearCache()}>
              <Eraser />
              清缓存
            </Button>
          </PermissionGate>
        </div>
      </section>

      <DataTable columns={columns} data={rows} loading={loading} />

      <Dialog open={editorOpen} onOpenChange={setEditorOpen}>
        <DialogContent className="max-w-3xl">
          <DialogHeader>
            <DialogTitle>{editingMenu ? "编辑菜单" : "新增菜单"}</DialogTitle>
            <DialogDescription>菜单、按钮权限和路由配置</DialogDescription>
          </DialogHeader>
          <MenuForm
            value={editingMenu}
            menus={menus}
            submitting={saving}
            onSubmit={saveMenu}
            onCancel={() => setEditorOpen(false)}
          />
        </DialogContent>
      </Dialog>
    </div>
  );
}
