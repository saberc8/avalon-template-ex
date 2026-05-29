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
import { PermissionGate } from "@/components/permission/permission-gate";
import { StatusBadge } from "@/components/system/status-badge";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
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

export function ConfigStorage() {
  const [items, setItems] = useState<StorageResp[]>([]);
  const [editing, setEditing] = useState<StorageResp | null>(null);
  const [form, setForm] = useState<StorageCommand>(emptyStorage);
  const [open, setOpen] = useState(false);

  const load = useCallback(async () => {
    try {
      setItems(await listStorage({ sort: ["sort,asc"] }));
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "存储列表加载失败");
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
          <div className="flex flex-wrap gap-1">
            <PermissionGate permissions={["system:storage:update"]}>
              <Button size="icon" variant="ghost" title="编辑" onClick={() => void openEditor(row.original.id)}>
                <Pencil />
              </Button>
            </PermissionGate>
            <PermissionGate permissions={["system:storage:setDefault"]}>
              <Button size="icon" variant="ghost" title="设为默认" onClick={() => void makeDefault(row.original.id)}>
                <CheckCircle2 />
              </Button>
            </PermissionGate>
            <PermissionGate permissions={["system:storage:delete"]}>
              <Button size="icon" variant="ghost" title="删除" onClick={() => void remove(row.original.id)}>
                <Trash2 />
              </Button>
            </PermissionGate>
          </div>
        )
      }
    ],
    []
  );

  async function openEditor(id?: number) {
    if (!id) {
      setEditing(null);
      setForm(emptyStorage);
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

  async function remove(id: number) {
    if (!window.confirm("确认删除该存储？")) return;
    await deleteStorage(id);
    await load();
    toast.success("存储已删除");
  }

  return (
    <div className="grid gap-4">
      <div className="flex justify-end gap-2">
        <Button variant="outline" onClick={() => void load()}>
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
      <DataTable columns={columns} data={items} />
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
            {(["name", "code", "accessKey", "secretKey", "endpoint", "region", "bucketName", "domain", "description"] as const).map((key) => (
              <Input
                key={key}
                value={String(form[key])}
                placeholder={key}
                onChange={(event) => setForm({ ...form, [key]: event.target.value })}
              />
            ))}
            <Input
              value={form.sort}
              type="number"
              onChange={(event) => setForm({ ...form, sort: Number(event.target.value) })}
            />
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
