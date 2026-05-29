"use client";

import { Download, Eye, RefreshCw } from "lucide-react";
import { useCallback, useEffect, useMemo, useState } from "react";
import type { ColumnDef } from "@tanstack/react-table";
import { toast } from "sonner";
import { exportLoginLog, exportOperationLog, getLog, listLog } from "@/api/monitor/log";
import { DataTable } from "@/components/table/data-table";
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
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue
} from "@/components/ui/select";
import { saveBlob } from "@/lib/download";
import type { LogDetailResp, LogResp } from "@/types/monitor";

export default function LogPage() {
  const [logs, setLogs] = useState<LogResp[]>([]);
  const [total, setTotal] = useState(0);
  const [page, setPage] = useState(1);
  const [description, setDescription] = useState("");
  const [module, setModule] = useState("");
  const [status, setStatus] = useState("all");
  const [detail, setDetail] = useState<LogDetailResp | null>(null);
  const [loading, setLoading] = useState(false);

  const query = useMemo(
    () => ({
      page,
      size: 10,
      description: description || undefined,
      module: module || undefined,
      status: status === "all" ? undefined : Number(status),
      sort: ["createTime,desc"]
    }),
    [description, module, page, status]
  );

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const result = await listLog(query);
      setLogs(result.list);
      setTotal(result.total);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "日志加载失败");
    } finally {
      setLoading(false);
    }
  }, [query]);

  useEffect(() => {
    void load();
  }, [load]);

  const columns = useMemo<ColumnDef<LogResp>[]>(
    () => [
      { accessorKey: "description", header: "描述" },
      { accessorKey: "module", header: "模块" },
      { accessorKey: "ip", header: "IP" },
      {
        header: "状态",
        cell: ({ row }) => <StatusBadge status={row.original.status === 1 ? 1 : 2} />
      },
      { accessorKey: "timeTaken", header: "耗时(ms)" },
      { accessorKey: "createTime", header: "时间" },
      {
        id: "actions",
        header: "操作",
        cell: ({ row }) => (
          <PermissionGate permissions={["monitor:log:get"]}>
            <Button size="icon" variant="ghost" title="详情" onClick={() => void openDetail(row.original.id)}>
              <Eye />
            </Button>
          </PermissionGate>
        )
      }
    ],
    []
  );

  async function openDetail(id: number) {
    try {
      setDetail(await getLog(id));
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "日志详情加载失败");
    }
  }

  async function exportLogs(kind: "login" | "operation") {
    try {
      const blob = kind === "login" ? await exportLoginLog(query) : await exportOperationLog(query);
      saveBlob(blob, kind === "login" ? "login_logs.csv" : "operation_logs.csv");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "日志导出失败");
    }
  }

  const pageCount = Math.max(1, Math.ceil(total / 10));

  return (
    <div className="mx-auto flex w-full max-w-7xl flex-col gap-4">
      <section className="flex flex-col gap-3 rounded-lg border bg-background p-4 lg:flex-row lg:items-end lg:justify-between">
        <div className="grid gap-3 md:grid-cols-3">
          <div className="grid gap-2">
            <span className="text-sm font-medium">描述</span>
            <Input value={description} onChange={(event) => setDescription(event.target.value)} />
          </div>
          <div className="grid gap-2">
            <span className="text-sm font-medium">模块</span>
            <Input value={module} onChange={(event) => setModule(event.target.value)} />
          </div>
          <div className="grid gap-2">
            <span className="text-sm font-medium">状态</span>
            <Select value={status} onValueChange={setStatus}>
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">全部</SelectItem>
                <SelectItem value="1">成功</SelectItem>
                <SelectItem value="2">失败</SelectItem>
              </SelectContent>
            </Select>
          </div>
        </div>
        <div className="flex flex-wrap gap-2">
          <Button variant="outline" onClick={() => void load()}>
            <RefreshCw />
            刷新
          </Button>
          <PermissionGate permissions={["monitor:log:export"]}>
            <Button variant="outline" onClick={() => void exportLogs("login")}>
              <Download />
              登录导出
            </Button>
            <Button variant="outline" onClick={() => void exportLogs("operation")}>
              <Download />
              操作导出
            </Button>
          </PermissionGate>
        </div>
      </section>
      <DataTable columns={columns} data={logs} loading={loading} />
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
      <Dialog open={!!detail} onOpenChange={(open) => !open && setDetail(null)}>
        <DialogContent className="max-w-3xl">
          <DialogHeader>
            <DialogTitle>日志详情</DialogTitle>
            <DialogDescription>{detail?.description}</DialogDescription>
          </DialogHeader>
          <div className="grid max-h-[70vh] gap-3 overflow-y-auto text-sm">
            {detail
              ? Object.entries(detail).map(([key, value]) => (
                  <div key={key} className="grid gap-1 rounded-md border p-3">
                    <div className="font-medium">{key}</div>
                    <pre className="whitespace-pre-wrap break-all text-xs text-muted-foreground">{String(value)}</pre>
                  </div>
                ))
              : null}
          </div>
        </DialogContent>
      </Dialog>
    </div>
  );
}
