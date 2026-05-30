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
import { TableActionButton, TableActions } from "@/components/table/table-actions";
import { PermissionGate } from "@/components/permission/permission-gate";
import { DictForm, DictItemForm } from "@/components/system/dict-form";
import { StatusBadge } from "@/components/system/status-badge";
import { Badge } from "@/components/ui/badge";
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
  const [dictLoading, setDictLoading] = useState(false);
  const [itemLoading, setItemLoading] = useState(false);
  const [dictDeleteTarget, setDictDeleteTarget] = useState<DictResp | null>(null);
  const [itemDeleteTarget, setItemDeleteTarget] = useState<DictItemResp | null>(null);
  const [deleteSubmitting, setDeleteSubmitting] = useState(false);

  const loadDicts = useCallback(async () => {
    setDictLoading(true);
    try {
      const data = await listDict({ description: keyword || undefined, sort: ["id,desc"] });
      setDicts(data);
      setSelectedDict((current) => {
        if (!data.length) {
          return null;
        }
        if (!current) {
          return data[0];
        }
        return data.find((dict) => dict.id === current.id) ?? data[0];
      });
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "字典列表加载失败");
    } finally {
      setDictLoading(false);
    }
  }, [keyword]);

  const loadItems = useCallback(async () => {
    if (!selectedDict) {
      setItems([]);
      return;
    }
    setItemLoading(true);
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
    } finally {
      setItemLoading(false);
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
      {
        header: "颜色",
        cell: ({ row }) => {
          const color = row.original.color;
          if (!color) {
            return "-";
          }
          return (
            <span className="inline-flex items-center gap-2">
              <span className="size-3 rounded-sm border" style={{ backgroundColor: color }} />
              {color}
            </span>
          );
        }
      },
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
            <PermissionGate permissions={["system:dict:item:update"]}>
              <TableActionButton icon={Pencil} label="编辑" onClick={() => void openItem(row.original.id)} />
            </PermissionGate>
            <PermissionGate permissions={["system:dict:item:delete"]}>
              <TableActionButton icon={Trash2} label="删除" destructive onClick={() => setItemDeleteTarget(row.original)} />
            </PermissionGate>
          </TableActions>
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

  async function confirmRemoveDict() {
    if (!dictDeleteTarget) return;
    setDeleteSubmitting(true);
    try {
      await deleteDict(dictDeleteTarget.id);
      setDictDeleteTarget(null);
      setSelectedDict(null);
      await loadDicts();
      toast.success("字典已删除");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "字典删除失败");
    } finally {
      setDeleteSubmitting(false);
    }
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

  async function confirmRemoveItem() {
    if (!itemDeleteTarget) return;
    setDeleteSubmitting(true);
    try {
      await deleteDictItem(itemDeleteTarget.id);
      setItemDeleteTarget(null);
      await loadItems();
      toast.success("字典项已删除");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "字典项删除失败");
    } finally {
      setDeleteSubmitting(false);
    }
  }

  async function clearCache() {
    if (!selectedDict) return;
    await clearDictCache(selectedDict.code);
    toast.success("字典缓存已清理");
  }

  return (
    <div
      className="mx-auto grid w-full max-w-7xl items-start gap-4 lg:grid-cols-[340px_1fr]"
      data-testid="dict-layout"
    >
      <section className="self-start rounded-lg border bg-background p-4" data-testid="dict-list-panel">
        <div className="mb-3 flex items-center justify-between">
          <div>
            <h2 className="text-base font-semibold">字典</h2>
            <p className="text-xs text-muted-foreground">{dicts.length} 个配置分组</p>
          </div>
          <PermissionGate permissions={["system:dict:create"]}>
            <Button size="sm" onClick={() => void openDict()}>
              <FilePlus2 />
              新增
            </Button>
          </PermissionGate>
        </div>
        <div className="mb-3 flex gap-2">
          <Input
            value={keyword}
            placeholder="搜索字典名称或描述"
            onChange={(event) => setKeyword(event.target.value)}
          />
          <Button variant="outline" size="sm" onClick={() => void loadDicts()} disabled={dictLoading}>
            <RefreshCw />
            刷新
          </Button>
        </div>
        <div className="grid gap-2">
          {dictLoading ? <div className="rounded-md border border-dashed p-6 text-center text-sm text-muted-foreground">加载中</div> : null}
          {!dictLoading && !dicts.length ? (
            <div className="rounded-md border border-dashed p-6 text-center text-sm text-muted-foreground">暂无字典</div>
          ) : null}
          {dicts.map((dict) => (
            <div
              key={dict.id}
              className={`group relative rounded-md border text-sm transition-colors ${selectedDict?.id === dict.id ? "border-primary bg-primary/5" : "bg-background hover:bg-muted/35"}`}
              data-testid={`dict-card-${dict.id}`}
            >
              <button
                type="button"
                aria-pressed={selectedDict?.id === dict.id}
                className="block w-full rounded-md p-3 pr-20 text-left outline-none focus-visible:ring-2 focus-visible:ring-ring"
                onClick={() => setSelectedDict(dict)}
              >
                <div className="flex min-w-0 items-center gap-2">
                  <div className="truncate font-medium">{dict.name}</div>
                  {dict.isSystem ? <Badge variant="secondary">系统</Badge> : null}
                </div>
                <div className="truncate text-xs text-muted-foreground">{dict.code}</div>
                {dict.description ? <div className="mt-1 line-clamp-2 text-xs text-muted-foreground">{dict.description}</div> : null}
              </button>
              <div
                className="pointer-events-none absolute right-2 top-2 flex gap-1 opacity-0 transition-opacity group-focus-within:pointer-events-auto group-focus-within:opacity-100 group-hover:pointer-events-auto group-hover:opacity-100"
                data-testid={`dict-card-actions-${dict.id}`}
              >
                <PermissionGate permissions={["system:dict:update"]}>
                  <Button
                    type="button"
                    size="icon"
                    variant="ghost"
                    aria-label={`编辑 ${dict.name}`}
                    title="编辑"
                    className="size-8 bg-background/85 shadow-sm hover:bg-muted"
                    onClick={() => void openDict(dict.id)}
                  >
                    <Pencil />
                  </Button>
                </PermissionGate>
                <PermissionGate permissions={["system:dict:delete"]}>
                  <Button
                    type="button"
                    size="icon"
                    variant="ghost"
                    aria-label={`删除 ${dict.name}`}
                    title="删除"
                    className="size-8 bg-background/85 text-destructive shadow-sm hover:bg-destructive/10 hover:text-destructive"
                    disabled={dict.isSystem}
                    onClick={() => setDictDeleteTarget(dict)}
                  >
                    <Trash2 />
                  </Button>
                </PermissionGate>
              </div>
            </div>
          ))}
        </div>
      </section>

      <section className="grid self-start content-start gap-4" data-testid="dict-items-panel">
        <div className="flex flex-col gap-3 rounded-lg border bg-background p-4 md:flex-row md:items-end md:justify-between">
          <div className="grid gap-2 md:w-80">
            <div>
              <span className="text-sm font-medium">{selectedDict?.name ?? "字典项"}</span>
              <p className="text-xs text-muted-foreground">{selectedDict?.code ?? "请选择左侧字典"}</p>
            </div>
            <Input
              value={itemKeyword}
              placeholder="搜索字典项标签或描述"
              disabled={!selectedDict}
              onChange={(event) => setItemKeyword(event.target.value)}
            />
          </div>
          <div className="flex flex-wrap gap-2">
            <Button variant="outline" onClick={() => void loadItems()} disabled={!selectedDict || itemLoading}>
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
        <DataTable columns={itemColumns} data={items} loading={itemLoading} emptyText={selectedDict ? "暂无字典项" : "请选择左侧字典"} />
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

      <Dialog open={!!dictDeleteTarget} onOpenChange={(open) => !open && setDictDeleteTarget(null)}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>删除字典</DialogTitle>
            <DialogDescription>确认删除“{dictDeleteTarget?.name}”？相关字典项也会被删除。</DialogDescription>
          </DialogHeader>
          <div className="flex justify-end gap-2">
            <Button variant="outline" onClick={() => setDictDeleteTarget(null)}>
              取消
            </Button>
            <Button variant="destructive" disabled={deleteSubmitting} onClick={() => void confirmRemoveDict()}>
              删除
            </Button>
          </div>
        </DialogContent>
      </Dialog>

      <Dialog open={!!itemDeleteTarget} onOpenChange={(open) => !open && setItemDeleteTarget(null)}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>删除字典项</DialogTitle>
            <DialogDescription>确认删除“{itemDeleteTarget?.label}”？</DialogDescription>
          </DialogHeader>
          <div className="flex justify-end gap-2">
            <Button variant="outline" onClick={() => setItemDeleteTarget(null)}>
              取消
            </Button>
            <Button variant="destructive" disabled={deleteSubmitting} onClick={() => void confirmRemoveItem()}>
              删除
            </Button>
          </div>
        </DialogContent>
      </Dialog>
    </div>
  );
}
