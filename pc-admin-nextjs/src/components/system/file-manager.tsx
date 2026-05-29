"use client";

import { Calculator, FolderPlus, Pencil, RefreshCw, Trash2, Upload } from "lucide-react";
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
import { PermissionGate } from "@/components/permission/permission-gate";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import type { FileItem, FileStatisticsResp } from "@/types/system";

export function FileManager() {
  const [files, setFiles] = useState<FileItem[]>([]);
  const [total, setTotal] = useState(0);
  const [page, setPage] = useState(1);
  const [keyword, setKeyword] = useState("");
  const [parentPath, setParentPath] = useState("");
  const [stats, setStats] = useState<FileStatisticsResp | null>(null);
  const [loading, setLoading] = useState(false);
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
          <button className="text-left font-medium hover:text-primary" onClick={() => openFile(row.original)}>
            {row.original.originalName}
          </button>
        )
      },
      {
        header: "大小",
        cell: ({ row }) => formatSize(row.original.size)
      },
      { accessorKey: "storageName", header: "存储" },
      { accessorKey: "createTime", header: "上传时间" },
      {
        id: "actions",
        header: "操作",
        cell: ({ row }) => (
          <div className="flex flex-wrap gap-1">
            <PermissionGate permissions={["system:file:update"]}>
              <Button size="icon" variant="ghost" title="重命名" onClick={() => void renameFile(row.original)}>
                <Pencil />
              </Button>
            </PermissionGate>
            <PermissionGate permissions={["system:file:calcDirSize"]}>
              <Button size="icon" variant="ghost" title="计算大小" onClick={() => void calculateSize(row.original)}>
                <Calculator />
              </Button>
            </PermissionGate>
            <PermissionGate permissions={["system:file:delete"]}>
              <Button size="icon" variant="ghost" title="删除" onClick={() => void removeFile(row.original.id)}>
                <Trash2 />
              </Button>
            </PermissionGate>
          </div>
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

  async function createFolder() {
    const name = window.prompt("文件夹名称");
    if (!name) {
      return;
    }
    try {
      await createDir(parentPath, name);
      await loadFiles();
      toast.success("文件夹已创建");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "文件夹创建失败");
    }
  }

  async function renameFile(file: FileItem) {
    const nextName = window.prompt("新名称", file.originalName);
    if (!nextName || nextName === file.originalName) {
      return;
    }
    try {
      await updateFile(file.id, nextName);
      await loadFiles();
      toast.success("文件已重命名");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "文件重命名失败");
    }
  }

  async function removeFile(id: number) {
    if (!window.confirm("确认删除该文件？")) {
      return;
    }
    try {
      await deleteFile([id]);
      await loadFiles();
      toast.success("文件已删除");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "文件删除失败");
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

  const pageCount = Math.max(1, Math.ceil(total / 12));

  return (
    <div className="grid gap-4">
      <section className="grid gap-3 rounded-lg border bg-background p-4 md:grid-cols-[1fr_auto] md:items-end">
        <div className="grid gap-3 md:grid-cols-2">
          <div className="grid gap-2">
            <span className="text-sm font-medium">文件名</span>
            <Input value={keyword} onChange={(event) => setKeyword(event.target.value)} />
          </div>
          <div className="grid gap-2">
            <span className="text-sm font-medium">目录</span>
            <Input value={parentPath} onChange={(event) => setParentPath(event.target.value)} />
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
            <Button variant="outline" onClick={() => void createFolder()}>
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
