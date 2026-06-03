"use client";

import { ChevronDown, ChevronRight, Download, FilePlus2, Pencil, RefreshCw, Trash2 } from "lucide-react";
import { useCallback, useEffect, useMemo, useState } from "react";
import type { ColumnDef } from "@tanstack/react-table";
import { toast } from "sonner";
import { addDept, deleteDept, exportDept, getDept, listDept, updateDept } from "@/api/system/dept";
import { DataTable } from "@/components/table/data-table";
import { TableActionButton, TableActions } from "@/components/table/table-actions";
import { PermissionGate } from "@/components/permission/permission-gate";
import { DeptForm } from "@/components/system/dept-form";
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
import { saveBlob } from "@/lib/download";
import { flattenVisibleTree } from "@/lib/tree";
import type { DeptCommand, DeptResp } from "@/types/system";

type DeptRow = DeptResp & { depth: number; hasChildren: boolean; expanded: boolean };

export default function DeptPage() {
  const [depts, setDepts] = useState<DeptResp[]>([]);
  const [description, setDescription] = useState("");
  const [status, setStatus] = useState("all");
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [editorOpen, setEditorOpen] = useState(false);
  const [editingDept, setEditingDept] = useState<DeptResp | null>(null);
  const [collapsedDeptIds, setCollapsedDeptIds] = useState<Set<number>>(new Set());

  const loadDepts = useCallback(async () => {
    setLoading(true);
    try {
      setDepts(
        await listDept({
          description: description || undefined,
          status: status === "all" ? undefined : Number(status)
        })
      );
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "部门列表加载失败");
    } finally {
      setLoading(false);
    }
  }, [description, status]);

  useEffect(() => {
    void loadDepts();
  }, [loadDepts]);

  const rows = useMemo<DeptRow[]>(
    () =>
      flattenVisibleTree(depts, collapsedDeptIds).map(({ node, depth, hasChildren, expanded }) => ({
        ...node,
        depth,
        hasChildren,
        expanded
      })),
    [collapsedDeptIds, depts]
  );

  const columns = useMemo<ColumnDef<DeptRow>[]>(
    () => [
      {
        header: "部门名称",
        cell: ({ row }) => (
          <div className="flex items-center gap-2" style={{ paddingLeft: row.original.depth * 20 }}>
            {row.original.hasChildren ? (
              <Button
                type="button"
                size="icon"
                variant="ghost"
                aria-expanded={row.original.expanded}
                aria-label={`${row.original.expanded ? "收起" : "展开"} ${row.original.name}`}
                className="size-7 shrink-0"
                onClick={() => toggleDept(row.original.id)}
              >
                {row.original.expanded ? <ChevronDown /> : <ChevronRight />}
              </Button>
            ) : (
              <span className="size-7 shrink-0" aria-hidden="true" />
            )}
            <span>{row.original.name}</span>
          </div>
        )
      },
      { accessorKey: "sort", header: "排序" },
      {
        header: "状态",
        cell: ({ row }) => <StatusBadge status={row.original.status} />
      },
      { accessorKey: "createTime", header: "创建时间" },
      {
        id: "actions",
        header: "操作",
        cell: ({ row }) => (
          <TableActions>
            <PermissionGate permissions={["system:dept:update"]}>
              <TableActionButton icon={Pencil} label="编辑" onClick={() => void openEditor(row.original.id)} />
            </PermissionGate>
            <PermissionGate permissions={["system:dept:delete"]}>
              <TableActionButton
                icon={Trash2}
                label="删除"
                destructive
                disabled={row.original.isSystem}
                onClick={() => void removeDept(row.original.id)}
              />
            </PermissionGate>
          </TableActions>
        )
      }
    ],
    []
  );

  function toggleDept(id: number) {
    setCollapsedDeptIds((current) => {
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
      setEditingDept(null);
      setEditorOpen(true);
      return;
    }
    try {
      setEditingDept(await getDept(id));
      setEditorOpen(true);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "部门详情加载失败");
    }
  }

  async function saveDept(command: DeptCommand) {
    setSaving(true);
    try {
      if (editingDept) {
        await updateDept(editingDept.id, command);
      } else {
        await addDept(command);
      }
      setEditorOpen(false);
      await loadDepts();
      toast.success("部门已保存");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "部门保存失败");
    } finally {
      setSaving(false);
    }
  }

  async function removeDept(id: number) {
    if (!window.confirm("确认删除该部门？")) {
      return;
    }
    try {
      await deleteDept(id);
      await loadDepts();
      toast.success("部门已删除");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "部门删除失败");
    }
  }

  async function handleExport() {
    try {
      saveBlob(
        await exportDept({
          description: description || undefined,
          status: status === "all" ? undefined : Number(status)
        }),
        "dept_export.csv"
      );
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "部门导出失败");
    }
  }

  return (
    <div className="mx-auto flex w-full max-w-7xl flex-col gap-4">
      <section className="flex flex-col gap-3 rounded-lg border bg-background p-4 md:flex-row md:items-end md:justify-between">
        <div className="grid gap-3 md:grid-cols-2 md:items-end">
          <div className="grid gap-2 md:w-72">
            <span className="text-sm font-medium">关键词</span>
            <Input value={description} onChange={(event) => setDescription(event.target.value)} />
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
          <Button variant="outline" onClick={() => void loadDepts()}>
            <RefreshCw />
            刷新
          </Button>
          <PermissionGate permissions={["system:dept:create"]}>
            <Button onClick={() => void openEditor()}>
              <FilePlus2 />
              新增
            </Button>
          </PermissionGate>
          <PermissionGate permissions={["system:dept:export"]}>
            <Button variant="outline" onClick={() => void handleExport()}>
              <Download />
              导出
            </Button>
          </PermissionGate>
        </div>
      </section>

      <DataTable columns={columns} data={rows} loading={loading} />

      <Dialog open={editorOpen} onOpenChange={setEditorOpen}>
        <DialogContent className="max-w-2xl">
          <DialogHeader>
            <DialogTitle>{editingDept ? "编辑部门" : "新增部门"}</DialogTitle>
            <DialogDescription>部门层级、状态和排序配置</DialogDescription>
          </DialogHeader>
          <DeptForm
            value={editingDept}
            depts={depts}
            submitting={saving}
            onSubmit={saveDept}
            onCancel={() => setEditorOpen(false)}
          />
        </DialogContent>
      </Dialog>
    </div>
  );
}
