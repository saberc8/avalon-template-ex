"use client";

import { ArrowUp, Calculator, FileText, FolderOpen, FolderPlus, Pencil, RefreshCw, Trash2, Upload } from "lucide-react";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { ColumnDef } from "@tanstack/react-table";
import { toast } from "sonner";
import {
  calcDirSize,
  createDir,
  deleteFile,
  getFileStatistics,
  listFile,
  updateFile,
  uploadFile
} from "@/api/system/file";
import { DataTable } from "@/components/table/data-table";
import { TableActionButton, TableActions } from "@/components/table/table-actions";
import { PermissionGate } from "@/components/permission/permission-gate";
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
import { Label } from "@/components/ui/label";
import type { FileItem, FileStatisticsResp } from "@/types/system";

export function FileManager() {
  const [files, setFiles] = useState<FileItem[]>([]);
  const [total, setTotal] = useState(0);
  const [page, setPage] = useState(1);
  const [keyword, setKeyword] = useState("");
  const [parentPath, setParentPath] = useState("");
  const [stats, setStats] = useState<FileStatisticsResp | null>(null);
  const [loading, setLoading] = useState(false);
  const [folderDialogOpen, setFolderDialogOpen] = useState(false);
  const [folderName, setFolderName] = useState("");
  const [renameTarget, setRenameTarget] = useState<FileItem | null>(null);
  const [renameValue, setRenameValue] = useState("");
  const [deleteTarget, setDeleteTarget] = useState<FileItem | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  const loadFiles = useCallback(async () => {
    setLoading(true);
    try {
      const [pageResult, statResult] = await Promise.all([
        listFile({
          page,
          size: 12,
          originalName: keyword || undefined,
          parentPath,
          sort: ["id,desc"]
        }),
        getFileStatistics()
      ]);
      setFiles(pageResult.list);
      setTotal(pageResult.total);
      setStats(statResult);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "文件列表加载失败");
    } finally {
      setLoading(false);
    }
  }, [keyword, page, parentPath]);

  useEffect(() => {
    void loadFiles();
  }, [loadFiles]);

  const columns = useMemo<ColumnDef<FileItem>[]>(
    () => [
      {
        header: "名称",
        cell: ({ row }) => (
          <button className="inline-flex items-center gap-2 text-left font-medium hover:text-primary" onClick={() => openFile(row.original)}>
            {row.original.type === 0 ? <FolderOpen className="size-4 text-amber-500" /> : <FileText className="size-4 text-muted-foreground" />}
            {row.original.originalName}
          </button>
        )
      },
      {
        header: "类型",
        cell: ({ row }) => <Badge variant="outline">{fileTypeLabel(row.original.type)}</Badge>
      },
      {
        header: "大小",
        cell: ({ row }) => (row.original.type === 0 && !row.original.size ? "-" : formatSize(row.original.size))
      },
      { accessorKey: "storageName", header: "存储" },
      { accessorKey: "createTime", header: "上传时间" },
      {
        id: "actions",
        header: "操作",
        cell: ({ row }) => (
          <TableActions>
            <PermissionGate permissions={["system:file:update"]}>
              <TableActionButton icon={Pencil} label="重命名" onClick={() => renameFile(row.original)} />
            </PermissionGate>
            <PermissionGate permissions={["system:file:calcDirSize"]}>
              <TableActionButton
                icon={Calculator}
                label="计算大小"
                disabled={row.original.type !== 0}
                onClick={() => void calculateSize(row.original)}
              />
            </PermissionGate>
            <PermissionGate permissions={["system:file:delete"]}>
              <TableActionButton icon={Trash2} label="删除" destructive onClick={() => removeFile(row.original)} />
            </PermissionGate>
          </TableActions>
        )
      }
    ],
    []
  );

  async function handleUpload(file: File) {
    const formData = new FormData();
    formData.append("file", file);
    formData.append("parentPath", parentPath);
    try {
      await uploadFile(formData);
      await loadFiles();
      toast.success("文件已上传");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "文件上传失败");
    }
  }

  function openCreateFolder() {
    setFolderName("");
    setFolderDialogOpen(true);
  }

  async function submitCreateFolder() {
    const name = folderName.trim();
    if (!name) {
      toast.error("请输入文件夹名称");
      return;
    }
    setSubmitting(true);
    try {
      await createDir(parentPath, name);
      setFolderDialogOpen(false);
      await loadFiles();
      toast.success("文件夹已创建");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "文件夹创建失败");
    } finally {
      setSubmitting(false);
    }
  }

  function renameFile(file: FileItem) {
    setRenameTarget(file);
    setRenameValue(file.originalName);
  }

  async function submitRename() {
    if (!renameTarget) {
      return;
    }
    const nextName = renameValue.trim();
    if (!nextName || nextName === renameTarget.originalName) {
      return;
    }
    setSubmitting(true);
    try {
      await updateFile(renameTarget.id, nextName);
      setRenameTarget(null);
      await loadFiles();
      toast.success("文件已重命名");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "文件重命名失败");
    } finally {
      setSubmitting(false);
    }
  }

  function removeFile(file: FileItem) {
    setDeleteTarget(file);
  }

  async function confirmRemoveFile() {
    if (!deleteTarget) {
      return;
    }
    setSubmitting(true);
    try {
      await deleteFile([deleteTarget.id]);
      setDeleteTarget(null);
      await loadFiles();
      toast.success("文件已删除");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "文件删除失败");
    } finally {
      setSubmitting(false);
    }
  }

  async function calculateSize(file: FileItem) {
    try {
      const result = await calcDirSize(file.id);
      toast.info(`${file.originalName}: ${formatSize(result.size)}`);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "大小计算失败");
    }
  }

  function openFile(file: FileItem) {
    if (file.type === 0) {
      setParentPath(file.path);
      setPage(1);
      return;
    }
    if (file.url) {
      window.open(file.url, "_blank", "noopener,noreferrer");
    }
  }

  function goUp() {
    if (!parentPath || parentPath === "/") {
      setParentPath("");
      setPage(1);
      return;
    }
    const parts = parentPath.split("/").filter(Boolean);
    parts.pop();
    setParentPath(parts.length ? `/${parts.join("/")}` : "");
    setPage(1);
  }

  const pageCount = Math.max(1, Math.ceil(total / 12));

  return (
    <div className="grid gap-4">
      <section className="grid gap-3 rounded-lg border bg-background p-4 md:grid-cols-[1fr_auto] md:items-end">
        <div className="grid gap-3 md:grid-cols-2">
          <div className="grid gap-2">
            <span className="text-sm font-medium">文件名</span>
            <Input value={keyword} placeholder="搜索文件名" onChange={(event) => setKeyword(event.target.value)} />
          </div>
          <div className="grid gap-2">
            <span className="text-sm font-medium">目录</span>
            <div className="flex gap-2">
              <Input value={parentPath || "/"} onChange={(event) => setParentPath(event.target.value === "/" ? "" : event.target.value)} />
              <Button variant="outline" size="sm" disabled={!parentPath} onClick={goUp}>
                <ArrowUp />
                上级
              </Button>
            </div>
          </div>
        </div>
        <div className="flex flex-wrap gap-2">
          <Button variant="outline" onClick={() => void loadFiles()}>
            <RefreshCw />
            刷新
          </Button>
          <PermissionGate permissions={["system:file:upload"]}>
            <Button onClick={() => inputRef.current?.click()}>
              <Upload />
              上传
            </Button>
            <input
              ref={inputRef}
              className="hidden"
              type="file"
              onChange={(event) => {
                const file = event.target.files?.[0];
                if (file) void handleUpload(file);
                event.target.value = "";
              }}
            />
          </PermissionGate>
          <PermissionGate permissions={["system:file:createDir"]}>
            <Button variant="outline" onClick={openCreateFolder}>
              <FolderPlus />
              新建文件夹
            </Button>
          </PermissionGate>
        </div>
      </section>
      <section className="grid gap-3 md:grid-cols-3">
        <div className="rounded-lg border bg-background p-4">
          <div className="text-sm text-muted-foreground">文件数</div>
          <div className="mt-1 text-2xl font-semibold">{stats?.number ?? 0}</div>
        </div>
        <div className="rounded-lg border bg-background p-4">
          <div className="text-sm text-muted-foreground">总大小</div>
          <div className="mt-1 text-2xl font-semibold">{formatSize(stats?.size ?? 0)}</div>
        </div>
        <div className="rounded-lg border bg-background p-4">
          <div className="text-sm text-muted-foreground">当前目录</div>
          <div className="mt-1 truncate text-lg font-semibold">{parentPath || "/"}</div>
        </div>
      </section>
      <DataTable columns={columns} data={files} loading={loading} />
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

      <Dialog open={folderDialogOpen} onOpenChange={setFolderDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>新建文件夹</DialogTitle>
            <DialogDescription>在 {parentPath || "/"} 下创建目录</DialogDescription>
          </DialogHeader>
          <form
            className="grid gap-4"
            onSubmit={(event) => {
              event.preventDefault();
              void submitCreateFolder();
            }}
          >
            <div className="grid gap-2">
              <Label htmlFor="folder-name">文件夹名称</Label>
              <Input id="folder-name" value={folderName} onChange={(event) => setFolderName(event.target.value)} autoFocus />
            </div>
            <div className="flex justify-end gap-2">
              <Button type="button" variant="outline" onClick={() => setFolderDialogOpen(false)}>
                取消
              </Button>
              <Button type="submit" disabled={submitting}>
                创建
              </Button>
            </div>
          </form>
        </DialogContent>
      </Dialog>

      <Dialog open={!!renameTarget} onOpenChange={(open) => !open && setRenameTarget(null)}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>重命名</DialogTitle>
            <DialogDescription>{renameTarget?.path}</DialogDescription>
          </DialogHeader>
          <form
            className="grid gap-4"
            onSubmit={(event) => {
              event.preventDefault();
              void submitRename();
            }}
          >
            <div className="grid gap-2">
              <Label htmlFor="rename-value">新名称</Label>
              <Input id="rename-value" value={renameValue} onChange={(event) => setRenameValue(event.target.value)} autoFocus />
            </div>
            <div className="flex justify-end gap-2">
              <Button type="button" variant="outline" onClick={() => setRenameTarget(null)}>
                取消
              </Button>
              <Button type="submit" disabled={submitting || !renameValue.trim() || renameValue.trim() === renameTarget?.originalName}>
                保存
              </Button>
            </div>
          </form>
        </DialogContent>
      </Dialog>

      <Dialog open={!!deleteTarget} onOpenChange={(open) => !open && setDeleteTarget(null)}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>删除文件</DialogTitle>
            <DialogDescription>确认删除“{deleteTarget?.originalName}”？此操作不可恢复。</DialogDescription>
          </DialogHeader>
          <div className="flex justify-end gap-2">
            <Button variant="outline" onClick={() => setDeleteTarget(null)}>
              取消
            </Button>
            <Button variant="destructive" disabled={submitting} onClick={() => void confirmRemoveFile()}>
              删除
            </Button>
          </div>
        </DialogContent>
      </Dialog>
    </div>
  );
}

function formatSize(size: number) {
  if (size >= 1024 * 1024) {
    return `${(size / 1024 / 1024).toFixed(1)} MB`;
  }
  if (size >= 1024) {
    return `${(size / 1024).toFixed(1)} KB`;
  }
  return `${size} B`;
}

function fileTypeLabel(type: number) {
  const labels: Record<number, string> = {
    0: "文件夹",
    1: "图片",
    2: "视频",
    3: "音频",
    4: "文档",
    5: "压缩包"
  };
  return labels[type] ?? "其他";
}
