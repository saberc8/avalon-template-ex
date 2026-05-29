"use client";

import { Eraser, FilePlus2, Pencil, RefreshCw, Trash2 } from "lucide-react";
import { useCallback, useEffect, useMemo, useState } from "react";
import type { ColumnDef } from "@tanstack/react-table";
import { toast } from "sonner";
import {
  addDict,
  addDictItem,
  clearDictCache,
  deleteDict,
  deleteDictItem,
  getDict,
  getDictItem,
  listDict,
  listDictItem,
  updateDict,
  updateDictItem
} from "@/api/system/dict";
import { DataTable } from "@/components/table/data-table";
import { PermissionGate } from "@/components/permission/permission-gate";
import { DictForm, DictItemForm } from "@/components/system/dict-form";
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
import type { DictCommand, DictItemCommand, DictItemResp, DictResp } from "@/types/system";

export default function DictPage() {
  const [dicts, setDicts] = useState<DictResp[]>([]);
  const [items, setItems] = useState<DictItemResp[]>([]);
  const [selectedDict, setSelectedDict] = useState<DictResp | null>(null);
  const [keyword, setKeyword] = useState("");
  const [itemKeyword, setItemKeyword] = useState("");
  const [dictOpen, setDictOpen] = useState(false);
  const [itemOpen, setItemOpen] = useState(false);
  const [editingDict, setEditingDict] = useState<DictResp | null>(null);
  const [editingItem, setEditingItem] = useState<DictItemResp | null>(null);

  const loadDicts = useCallback(async () => {
    try {
      const data = await listDict({ description: keyword || undefined, sort: ["id,desc"] });
      setDicts(data);
      setSelectedDict((current) => current ?? data[0] ?? null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "字典列表加载失败");
    }
  }, [keyword]);

  const loadItems = useCallback(async () => {
    if (!selectedDict) {
      setItems([]);
      return;
    }
    try {
      const result = await listDictItem({
        page: 1,
        size: 50,
        dictId: selectedDict.id,
        description: itemKeyword || undefined,
        sort: ["sort,asc"]
      });
      setItems(result.list);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "字典项加载失败");
    }
  }, [itemKeyword, selectedDict]);

  useEffect(() => {
    void loadDicts();
  }, [loadDicts]);

  useEffect(() => {
    void loadItems();
  }, [loadItems]);

  const itemColumns = useMemo<ColumnDef<DictItemResp>[]>(
    () => [
      { accessorKey: "label", header: "标签" },
      { accessorKey: "value", header: "值" },
      { accessorKey: "color", header: "颜色" },
      {
        header: "状态",
        cell: ({ row }) => <StatusBadge status={row.original.status} />
      },
      { accessorKey: "sort", header: "排序" },
      {
        id: "actions",
        header: "操作",
        cell: ({ row }) => (
          <div className="flex flex-wrap gap-1">
            <PermissionGate permissions={["system:dict:item:update"]}>
              <Button size="icon" variant="ghost" title="编辑" onClick={() => void openItem(row.original.id)}>
                <Pencil />
              </Button>
            </PermissionGate>
            <PermissionGate permissions={["system:dict:item:delete"]}>
              <Button size="icon" variant="ghost" title="删除" onClick={() => void removeItem(row.original.id)}>
                <Trash2 />
              </Button>
            </PermissionGate>
          </div>
        )
      }
    ],
    []
  );

  async function openDict(id?: number) {
    setEditingDict(id ? await getDict(id) : null);
    setDictOpen(true);
  }

  async function saveDict(command: DictCommand) {
    try {
      if (editingDict) await updateDict(editingDict.id, command);
      else await addDict(command);
      setDictOpen(false);
      await loadDicts();
      toast.success("字典已保存");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "字典保存失败");
    }
  }

  async function removeDict(id: number) {
    if (!window.confirm("确认删除该字典？")) return;
    await deleteDict(id);
    setSelectedDict(null);
    await loadDicts();
    toast.success("字典已删除");
  }

  async function openItem(id?: number) {
    setEditingItem(id ? await getDictItem(id) : null);
    setItemOpen(true);
  }

  async function saveItem(command: DictItemCommand) {
    try {
      if (editingItem) await updateDictItem(editingItem.id, command);
      else await addDictItem(command);
      setItemOpen(false);
      await loadItems();
      toast.success("字典项已保存");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "字典项保存失败");
    }
  }

  async function removeItem(id: number) {
    if (!window.confirm("确认删除该字典项？")) return;
    await deleteDictItem(id);
    await loadItems();
    toast.success("字典项已删除");
  }

  async function clearCache() {
    if (!selectedDict) return;
    await clearDictCache(selectedDict.code);
    toast.success("字典缓存已清理");
  }

  return (
    <div className="mx-auto grid w-full max-w-7xl gap-4 lg:grid-cols-[320px_1fr]">
      <section className="rounded-lg border bg-background p-4">
        <div className="mb-3 flex items-center justify-between">
          <h2 className="text-base font-semibold">字典</h2>
          <PermissionGate permissions={["system:dict:create"]}>
            <Button size="sm" onClick={() => void openDict()}>
              <FilePlus2 />
              新增
            </Button>
          </PermissionGate>
        </div>
        <div className="mb-3 flex gap-2">
          <Input value={keyword} onChange={(event) => setKeyword(event.target.value)} />
          <Button variant="outline" size="icon" onClick={() => void loadDicts()}>
            <RefreshCw />
          </Button>
        </div>
        <div className="grid gap-2">
          {dicts.map((dict) => (
            <button
              key={dict.id}
              className={`rounded-md border p-3 text-left text-sm ${selectedDict?.id === dict.id ? "border-primary bg-primary/5" : "bg-background"}`}
              onClick={() => setSelectedDict(dict)}
            >
              <div className="font-medium">{dict.name}</div>
              <div className="text-xs text-muted-foreground">{dict.code}</div>
              <div className="mt-2 flex gap-1">
                <PermissionGate permissions={["system:dict:update"]}>
                  <Button size="sm" variant="ghost" onClick={(event) => { event.stopPropagation(); void openDict(dict.id); }}>
                    编辑
                  </Button>
                </PermissionGate>
                <PermissionGate permissions={["system:dict:delete"]}>
                  <Button size="sm" variant="ghost" disabled={dict.isSystem} onClick={(event) => { event.stopPropagation(); void removeDict(dict.id); }}>
                    删除
                  </Button>
                </PermissionGate>
              </div>
            </button>
          ))}
        </div>
      </section>

      <section className="grid gap-4">
        <div className="flex flex-col gap-3 rounded-lg border bg-background p-4 md:flex-row md:items-end md:justify-between">
          <div className="grid gap-2 md:w-80">
            <span className="text-sm font-medium">{selectedDict?.name ?? "字典项"}</span>
            <Input value={itemKeyword} onChange={(event) => setItemKeyword(event.target.value)} />
          </div>
          <div className="flex flex-wrap gap-2">
            <Button variant="outline" onClick={() => void loadItems()}>
              <RefreshCw />
              刷新
            </Button>
            <PermissionGate permissions={["system:dict:item:create"]}>
              <Button disabled={!selectedDict} onClick={() => void openItem()}>
                <FilePlus2 />
                新增项
              </Button>
            </PermissionGate>
            <PermissionGate permissions={["system:dict:item:clearCache"]}>
              <Button variant="outline" disabled={!selectedDict} onClick={() => void clearCache()}>
                <Eraser />
                清缓存
              </Button>
            </PermissionGate>
          </div>
        </div>
        <DataTable columns={itemColumns} data={items} />
      </section>

      <Dialog open={dictOpen} onOpenChange={setDictOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{editingDict ? "编辑字典" : "新增字典"}</DialogTitle>
            <DialogDescription>字典名称、编码和描述</DialogDescription>
          </DialogHeader>
          <DictForm value={editingDict} onSubmit={saveDict} onCancel={() => setDictOpen(false)} />
        </DialogContent>
      </Dialog>

      <Dialog open={itemOpen} onOpenChange={setItemOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{editingItem ? "编辑字典项" : "新增字典项"}</DialogTitle>
            <DialogDescription>{selectedDict?.name}</DialogDescription>
          </DialogHeader>
          <DictItemForm
            dictId={selectedDict?.id ?? 0}
            value={editingItem}
            onSubmit={saveItem}
            onCancel={() => setItemOpen(false)}
          />
        </DialogContent>
      </Dialog>
    </div>
  );
}
