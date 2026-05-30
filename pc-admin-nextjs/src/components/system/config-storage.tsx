"use client";

import { CheckCircle2, FilePlus2, Pencil, RefreshCw, Trash2 } from "lucide-react";
import { useCallback, useEffect, useMemo, useState } from "react";
import type { ColumnDef } from "@tanstack/react-table";
import { toast } from "sonner";
import {
  addStorage,
  deleteStorage,
  getStorage,
  listStorage,
  setDefaultStorage,
  updateStorage
} from "@/api/system/storage";
import { DataTable } from "@/components/table/data-table";
import { TableActionButton, TableActions } from "@/components/table/table-actions";
import { PermissionGate } from "@/components/permission/permission-gate";
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
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue
} from "@/components/ui/select";
import { Switch } from "@/components/ui/switch";
import type { StorageCommand, StorageResp } from "@/types/system";

const emptyStorage: StorageCommand = {
  name: "",
  code: "",
  type: 1,
  accessKey: "",
  secretKey: "",
  endpoint: "",
  region: "",
  bucketName: "",
  domain: "",
  description: "",
  isDefault: false,
  sort: 1,
  status: 1
};

const storageTextFields: Array<{ key: keyof Pick<StorageCommand, "name" | "code" | "accessKey" | "secretKey" | "endpoint" | "region" | "bucketName" | "domain" | "description">; label: string; placeholder?: string }> = [
  { key: "name", label: "存储名称", placeholder: "例如：本地存储" },
  { key: "code", label: "存储编码", placeholder: "例如：LOCAL" },
  { key: "accessKey", label: "Access Key" },
  { key: "secretKey", label: "Secret Key" },
  { key: "endpoint", label: "Endpoint" },
  { key: "region", label: "Region" },
  { key: "bucketName", label: "Bucket / 本地目录", placeholder: "./data/file/" },
  { key: "domain", label: "访问域名", placeholder: "/file/" },
  { key: "description", label: "描述" }
];

export function ConfigStorage() {
  const [items, setItems] = useState<StorageResp[]>([]);
  const [editing, setEditing] = useState<StorageResp | null>(null);
  const [form, setForm] = useState<StorageCommand>(emptyStorage);
  const [open, setOpen] = useState(false);
  const [loading, setLoading] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<StorageResp | null>(null);
  const [deleting, setDeleting] = useState(false);

  const load = useCallback(async () => {
    setLoading(true);
    try {
      setItems(await listStorage({ sort: ["sort,asc"] }));
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "存储列表加载失败");
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void load();
  }, [load]);

  const columns = useMemo<ColumnDef<StorageResp>[]>(
    () => [
      { accessorKey: "name", header: "名称" },
      { accessorKey: "code", header: "编码" },
      {
        header: "类型",
        cell: ({ row }) => (row.original.type === 1 ? "本地" : "对象存储")
      },
      { accessorKey: "sort", header: "排序" },
      {
        header: "默认",
        cell: ({ row }) => (row.original.isDefault ? "是" : "否")
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
            <PermissionGate permissions={["system:storage:update"]}>
              <TableActionButton icon={Pencil} label="编辑" onClick={() => void openEditor(row.original.id)} />
            </PermissionGate>
            <PermissionGate permissions={["system:storage:setDefault"]}>
              <TableActionButton
                icon={CheckCircle2}
                label="设为默认"
                disabled={row.original.isDefault}
                onClick={() => void makeDefault(row.original.id)}
              />
            </PermissionGate>
            <PermissionGate permissions={["system:storage:delete"]}>
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
      setForm({ ...emptyStorage });
      setOpen(true);
      return;
    }
    const detail = await getStorage(id);
    setEditing(detail);
    setForm(storageToCommand(detail));
    setOpen(true);
  }

  async function save() {
    try {
      if (editing) {
        await updateStorage(editing.id, form);
      } else {
        await addStorage(form);
      }
      setOpen(false);
      await load();
      toast.success("存储配置已保存");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "存储保存失败");
    }
  }

  async function makeDefault(id: number) {
    await setDefaultStorage(id);
    await load();
    toast.success("默认存储已更新");
  }

  async function confirmRemove() {
    if (!deleteTarget) return;
    setDeleting(true);
    try {
      await deleteStorage(deleteTarget.id);
      setDeleteTarget(null);
      await load();
      toast.success("存储已删除");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "存储删除失败");
    } finally {
      setDeleting(false);
    }
  }

  return (
    <div className="grid gap-4">
      <div className="flex flex-col gap-3 rounded-lg border bg-background p-4 md:flex-row md:items-center md:justify-between">
        <div>
          <h2 className="text-base font-semibold">存储配置</h2>
          <p className="mt-1 text-sm text-muted-foreground">{items.length} 个存储端，默认存储用于上传和文件访问</p>
        </div>
        <div className="flex justify-end gap-2">
          <Button variant="outline" onClick={() => void load()} disabled={loading}>
            <RefreshCw />
            刷新
          </Button>
          <PermissionGate permissions={["system:storage:create"]}>
            <Button onClick={() => void openEditor()}>
              <FilePlus2 />
              新增
            </Button>
          </PermissionGate>
        </div>
      </div>
      <DataTable columns={columns} data={items} loading={loading} emptyText="暂无存储配置" />
      <Dialog open={open} onOpenChange={setOpen}>
        <DialogContent className="max-w-3xl">
          <DialogHeader>
            <DialogTitle>{editing ? "编辑存储" : "新增存储"}</DialogTitle>
          </DialogHeader>
          <form
            className="grid gap-3 md:grid-cols-2"
            onSubmit={(event) => {
              event.preventDefault();
              void save();
            }}
          >
            <div className="grid gap-2">
              <Label>存储类型</Label>
              <Select value={String(form.type)} onValueChange={(type) => setForm({ ...form, type: Number(type) })}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="1">本地存储</SelectItem>
                  <SelectItem value="2">对象存储</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="grid gap-2">
              <Label>状态</Label>
              <Select value={String(form.status)} onValueChange={(status) => setForm({ ...form, status: Number(status) })}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="1">启用</SelectItem>
                  <SelectItem value="2">禁用</SelectItem>
                </SelectContent>
              </Select>
            </div>
            {storageTextFields.map((field) => (
              <div key={field.key} className="grid gap-2">
                <Label>{field.label}</Label>
                <Input
                  value={String(form[field.key])}
                  placeholder={field.placeholder}
                  onChange={(event) => setForm({ ...form, [field.key]: event.target.value })}
                />
              </div>
            ))}
            <div className="grid gap-2">
              <Label>排序</Label>
              <Input
                value={form.sort}
                type="number"
                onChange={(event) => setForm({ ...form, sort: Number(event.target.value) })}
              />
            </div>
            <label className="flex items-center justify-between rounded-md border p-3 text-sm">
              默认存储
              <Switch checked={form.isDefault} onCheckedChange={(isDefault) => setForm({ ...form, isDefault })} />
            </label>
            <div className="col-span-full flex justify-end gap-2">
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
            <DialogTitle>删除存储</DialogTitle>
            <DialogDescription>确认删除“{deleteTarget?.name}”？已关联文件的存储通常不能删除。</DialogDescription>
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

function storageToCommand(storage: StorageResp): StorageCommand {
  return {
    name: storage.name,
    code: storage.code,
    type: storage.type,
    accessKey: storage.accessKey,
    secretKey: storage.secretKey,
    endpoint: storage.endpoint,
    region: storage.region,
    bucketName: storage.bucketName,
    domain: storage.domain,
    description: storage.description,
    isDefault: storage.isDefault,
    sort: storage.sort,
    status: storage.status
  };
}
