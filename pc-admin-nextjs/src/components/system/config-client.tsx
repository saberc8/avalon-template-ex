"use client";

import { FilePlus2, Pencil, RefreshCw, Trash2 } from "lucide-react";
import { useCallback, useEffect, useMemo, useState } from "react";
import type { ColumnDef } from "@tanstack/react-table";
import { toast } from "sonner";
import { addClient, deleteClient, getClient, listClient, updateClient } from "@/api/system/client";
import { DataTable } from "@/components/table/data-table";
import { TableActionButton, TableActions } from "@/components/table/table-actions";
import { PermissionGate } from "@/components/permission/permission-gate";
import { StatusBadge } from "@/components/system/status-badge";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue
} from "@/components/ui/select";
import type { ClientCommand, ClientResp } from "@/types/system";

const emptyClient: ClientCommand = {
  clientType: "PC",
  authType: ["ACCOUNT"],
  activeTimeout: 1800,
  timeout: 86400,
  status: 1
};

const clientTypeLabels: Record<string, string> = {
  PC: "桌面端",
  ANDROID: "安卓",
  XCX: "小程序"
};

const authTypeLabels: Record<string, string> = {
  ACCOUNT: "账号密码",
  EMAIL: "邮箱",
  PHONE: "手机号",
  SOCIAL: "三方登录"
};

export function ConfigClient() {
  const [clients, setClients] = useState<ClientResp[]>([]);
  const [total, setTotal] = useState(0);
  const [page, setPage] = useState(1);
  const [editing, setEditing] = useState<ClientResp | null>(null);
  const [form, setForm] = useState<ClientCommand>(emptyClient);
  const [open, setOpen] = useState(false);
  const [loading, setLoading] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<ClientResp | null>(null);
  const [deleting, setDeleting] = useState(false);

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const result = await listClient({ page, size: 10, sort: ["id,desc"] });
      setClients(result.list);
      setTotal(result.total);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "客户端列表加载失败");
    } finally {
      setLoading(false);
    }
  }, [page]);

  useEffect(() => {
    void load();
  }, [load]);

  const columns = useMemo<ColumnDef<ClientResp>[]>(
    () => [
      { accessorKey: "clientId", header: "客户端 ID" },
      {
        header: "类型",
        cell: ({ row }) => clientTypeLabels[row.original.clientType] ?? row.original.clientType
      },
      {
        header: "认证方式",
        cell: ({ row }) => row.original.authType.map((item) => authTypeLabels[item] ?? item).join("、")
      },
      {
        header: "状态",
        cell: ({ row }) => <StatusBadge status={row.original.status} />
      },
      {
        id: "actions",
        header: "操作",
        cell: ({ row }) => (
          <TableActions>
            <PermissionGate permissions={["system:client:update"]}>
              <TableActionButton icon={Pencil} label="编辑" onClick={() => void openEditor(row.original.id)} />
            </PermissionGate>
            <PermissionGate permissions={["system:client:delete"]}>
              <TableActionButton icon={Trash2} label="删除" destructive onClick={() => setDeleteTarget(row.original)} />
            </PermissionGate>
          </TableActions>
        )
      }
    ],
    []
  );

  async function openEditor(id?: number) {
    if (!id) {
      setEditing(null);
      setForm({ ...emptyClient });
      setOpen(true);
      return;
    }
    const detail = await getClient(id);
    setEditing(detail);
    setForm({
      clientType: detail.clientType,
      authType: detail.authType,
      activeTimeout: detail.activeTimeout,
      timeout: detail.timeout,
      status: detail.status
    });
    setOpen(true);
  }

  async function save() {
    try {
      if (editing) {
        await updateClient(editing.id, form);
      } else {
        await addClient(form);
      }
      setOpen(false);
      await load();
      toast.success("客户端已保存");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "客户端保存失败");
    }
  }

  async function confirmRemove() {
    if (!deleteTarget) return;
    setDeleting(true);
    try {
      await deleteClient(deleteTarget.id);
      setDeleteTarget(null);
      await load();
      toast.success("客户端已删除");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "客户端删除失败");
    } finally {
      setDeleting(false);
    }
  }

  const pageCount = Math.max(1, Math.ceil(total / 10));

  return (
    <div className="grid gap-4">
      <div className="flex flex-col gap-3 rounded-lg border bg-background p-4 md:flex-row md:items-center md:justify-between">
        <div>
          <h2 className="text-base font-semibold">客户端配置</h2>
          <p className="mt-1 text-sm text-muted-foreground">管理不同终端的认证方式和会话有效期</p>
        </div>
        <div className="flex justify-end gap-2">
          <Button variant="outline" onClick={() => void load()} disabled={loading}>
            <RefreshCw />
            刷新
          </Button>
          <PermissionGate permissions={["system:client:create"]}>
            <Button onClick={() => void openEditor()}>
              <FilePlus2 />
              新增
            </Button>
          </PermissionGate>
        </div>
      </div>
      <DataTable columns={columns} data={clients} loading={loading} emptyText="暂无客户端配置" />
      <div className="flex items-center justify-end gap-2 text-sm">
        <span className="text-muted-foreground">
          第 {page} / {pageCount} 页，共 {total} 条
        </span>
        <Button variant="outline" size="sm" disabled={page <= 1} onClick={() => setPage(page - 1)}>
          上一页
        </Button>
        <Button variant="outline" size="sm" disabled={page >= pageCount} onClick={() => setPage(page + 1)}>
          下一页
        </Button>
      </div>
      <Dialog open={open} onOpenChange={setOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{editing ? "编辑客户端" : "新增客户端"}</DialogTitle>
          </DialogHeader>
          <form
            className="grid gap-4"
            onSubmit={(event) => {
              event.preventDefault();
              void save();
            }}
          >
            <div className="grid gap-2">
              <Label>客户端类型</Label>
              <Select value={form.clientType} onValueChange={(clientType) => setForm({ ...form, clientType })}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="PC">桌面端</SelectItem>
                  <SelectItem value="ANDROID">安卓</SelectItem>
                  <SelectItem value="XCX">小程序</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="grid gap-2 rounded-md border p-3">
              <Label>认证方式</Label>
              <div className="grid gap-2 sm:grid-cols-2">
                {["ACCOUNT", "EMAIL", "PHONE", "SOCIAL"].map((authType) => (
                  <label key={authType} className="flex items-center gap-2 text-sm">
                    <Checkbox
                      checked={form.authType.includes(authType)}
                      onCheckedChange={(checked) =>
                        setForm({
                          ...form,
                          authType: checked
                            ? [...form.authType, authType]
                            : form.authType.filter((item) => item !== authType)
                        })
                      }
                    />
                    {authTypeLabels[authType]}
                  </label>
                ))}
              </div>
            </div>
            <div className="grid gap-2">
              <Label>活跃超时时间（秒）</Label>
              <Input
                value={form.activeTimeout}
                type="number"
                onChange={(event) => setForm({ ...form, activeTimeout: Number(event.target.value) })}
              />
            </div>
            <div className="grid gap-2">
              <Label>登录有效期（秒）</Label>
              <Input
                value={form.timeout}
                type="number"
                onChange={(event) => setForm({ ...form, timeout: Number(event.target.value) })}
              />
            </div>
            <div className="grid gap-2">
              <Label>状态</Label>
              <Select
                value={String(form.status)}
                onValueChange={(status) => setForm({ ...form, status: Number(status) })}
              >
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="1">启用</SelectItem>
                  <SelectItem value="2">禁用</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="flex justify-end gap-2">
              <Button type="button" variant="outline" onClick={() => setOpen(false)}>
                取消
              </Button>
              <Button type="submit">保存</Button>
            </div>
          </form>
        </DialogContent>
      </Dialog>
      <Dialog open={!!deleteTarget} onOpenChange={(dialogOpen) => !dialogOpen && setDeleteTarget(null)}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>删除客户端</DialogTitle>
            <DialogDescription>确认删除“{deleteTarget?.clientId}”？该客户端将不能继续用于登录。</DialogDescription>
          </DialogHeader>
          <div className="flex justify-end gap-2">
            <Button variant="outline" onClick={() => setDeleteTarget(null)}>
              取消
            </Button>
            <Button variant="destructive" disabled={deleting} onClick={() => void confirmRemove()}>
              删除
            </Button>
          </div>
        </DialogContent>
      </Dialog>
    </div>
  );
}
