"use client";

import { FilePlus2, Pencil, Play, RefreshCw, ScrollText, Trash2 } from "lucide-react";
import { useCallback, useEffect, useMemo, useState } from "react";
import type { ColumnDef } from "@tanstack/react-table";
import { toast } from "sonner";
import {
  addJob,
  deleteJob,
  getJob,
  listJob,
  listJobLog,
  runJob,
  updateJob,
  updateJobStatus
} from "@/api/schedule/job";
import { JobForm } from "@/components/schedule/job-form";
import { PermissionGate } from "@/components/permission/permission-gate";
import { StatusBadge } from "@/components/system/status-badge";
import { DataTable } from "@/components/table/data-table";
import { TableActionButton, TableActions } from "@/components/table/table-actions";
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
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue
} from "@/components/ui/select";
import type { JobCommand, JobLogResp, JobResp } from "@/types/schedule";

const taskTypeText: Record<number, string> = {
  1: "HTTP",
  2: "内置"
};

const logStatusText: Record<number, string> = {
  3: "运行中",
  4: "成功",
  5: "失败",
  6: "死信"
};

export default function ScheduleJobPage() {
  const [jobs, setJobs] = useState<JobResp[]>([]);
  const [total, setTotal] = useState(0);
  const [page, setPage] = useState(1);
  const [keyword, setKeyword] = useState("");
  const [status, setStatus] = useState("all");
  const [taskType, setTaskType] = useState("all");
  const [loading, setLoading] = useState(false);
  const [open, setOpen] = useState(false);
  const [editing, setEditing] = useState<JobResp | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<JobResp | null>(null);
  const [logsOpen, setLogsOpen] = useState(false);
  const [logJob, setLogJob] = useState<JobResp | null>(null);
  const [logs, setLogs] = useState<JobLogResp[]>([]);
  const [logsLoading, setLogsLoading] = useState(false);

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const result = await listJob({
        page,
        size: 10,
        description: keyword || undefined,
        status: status === "all" ? undefined : Number(status),
        taskType: taskType === "all" ? undefined : Number(taskType),
        sort: ["id,desc"]
      });
      setJobs(result.list);
      setTotal(result.total);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "定时任务加载失败");
    } finally {
      setLoading(false);
    }
  }, [keyword, page, status, taskType]);

  useEffect(() => {
    void load();
  }, [load]);

  const columns = useMemo<ColumnDef<JobResp>[]>(
    () => [
      {
        header: "任务",
        cell: ({ row }) => (
          <div className="min-w-48">
            <div className="font-medium">{row.original.name}</div>
            <div className="text-xs text-muted-foreground">{row.original.groupName}</div>
          </div>
        )
      },
      {
        header: "类型",
        cell: ({ row }) => <Badge variant="secondary">{taskTypeText[row.original.taskType]}</Badge>
      },
      { accessorKey: "cronExpression", header: "Cron" },
      {
        header: "目标",
        cell: ({ row }) => (
          <span className="block max-w-96 truncate">
            {row.original.taskType === 1 ? row.original.httpUrl : row.original.builtinKey}
          </span>
        )
      },
      {
        header: "状态",
        cell: ({ row }) => <StatusBadge status={row.original.status} />
      },
      { accessorKey: "nextTriggerTime", header: "下次触发" },
      {
        id: "actions",
        header: "操作",
        cell: ({ row }) => (
          <TableActions>
            <PermissionGate permissions={["schedule:job:run"]}>
              <TableActionButton icon={Play} label="执行" onClick={() => void run(row.original)} />
            </PermissionGate>
            <PermissionGate permissions={["schedule:job:updateStatus"]}>
              <TableActionButton
                icon={RefreshCw}
                label={row.original.status === 1 ? "禁用" : "启用"}
                onClick={() => void toggleStatus(row.original)}
              />
            </PermissionGate>
            <PermissionGate permissions={["schedule:job:log:list"]}>
              <TableActionButton icon={ScrollText} label="日志" onClick={() => void openLogs(row.original)} />
            </PermissionGate>
            <PermissionGate permissions={["schedule:job:update"]}>
              <TableActionButton icon={Pencil} label="编辑" onClick={() => void openEditor(row.original.id)} />
            </PermissionGate>
            <PermissionGate permissions={["schedule:job:delete"]}>
              <TableActionButton icon={Trash2} label="删除" destructive onClick={() => setDeleteTarget(row.original)} />
            </PermissionGate>
          </TableActions>
        )
      }
    ],
    []
  );

  const logColumns = useMemo<ColumnDef<JobLogResp>[]>(
    () => [
      { accessorKey: "attempt", header: "次数" },
      {
        header: "状态",
        cell: ({ row }) => <Badge variant={row.original.status === 4 ? "secondary" : "destructive"}>{logStatusText[row.original.status] ?? row.original.status}</Badge>
      },
      { accessorKey: "executor", header: "执行器" },
      { accessorKey: "responseStatus", header: "响应码" },
      { accessorKey: "timeTaken", header: "耗时(ms)" },
      { accessorKey: "startTime", header: "开始时间" },
      {
        header: "错误",
        cell: ({ row }) => <span className="block max-w-80 truncate">{row.original.errorMsg || "-"}</span>
      }
    ],
    []
  );

  async function openEditor(id?: number) {
    setEditing(id ? await getJob(id) : null);
    setOpen(true);
  }

  async function save(command: JobCommand) {
    setSubmitting(true);
    try {
      if (editing) await updateJob(editing.id, command);
      else await addJob(command);
      setOpen(false);
      await load();
      toast.success("定时任务已保存");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "定时任务保存失败");
    } finally {
      setSubmitting(false);
    }
  }

  async function toggleStatus(job: JobResp) {
    await updateJobStatus(job.id, job.status === 1 ? 2 : 1);
    await load();
    toast.success(job.status === 1 ? "任务已禁用" : "任务已启用");
  }

  async function run(job: JobResp) {
    await runJob(job.id);
    toast.success("已创建手动执行触发");
    if (logJob?.id === job.id) await loadLogs(job);
  }

  async function confirmRemove() {
    if (!deleteTarget) return;
    await deleteJob(deleteTarget.id);
    setDeleteTarget(null);
    await load();
    toast.success("定时任务已删除");
  }

  async function openLogs(job: JobResp) {
    setLogJob(job);
    setLogsOpen(true);
    await loadLogs(job);
  }

  async function loadLogs(job = logJob) {
    if (!job) return;
    setLogsLoading(true);
    try {
      const result = await listJobLog(job.id, { page: 1, size: 20 });
      setLogs(result.list);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "执行日志加载失败");
    } finally {
      setLogsLoading(false);
    }
  }

  const pageCount = Math.max(1, Math.ceil(total / 10));

  return (
    <div className="mx-auto grid w-full max-w-7xl gap-4">
      <section className="rounded-lg border bg-background p-4">
        <div className="flex flex-col gap-3 lg:flex-row lg:items-end lg:justify-between">
          <div className="grid gap-2 sm:grid-cols-[minmax(220px,1fr)_150px_150px]">
            <Input value={keyword} placeholder="搜索任务名称、分组或描述" onChange={(event) => setKeyword(event.target.value)} />
            <Select value={taskType} onValueChange={setTaskType}>
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">全部类型</SelectItem>
                <SelectItem value="1">HTTP</SelectItem>
                <SelectItem value="2">内置</SelectItem>
              </SelectContent>
            </Select>
            <Select value={status} onValueChange={setStatus}>
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">全部状态</SelectItem>
                <SelectItem value="1">启用</SelectItem>
                <SelectItem value="2">禁用</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div className="flex justify-end gap-2">
            <Button variant="outline" onClick={() => void load()} disabled={loading}>
              <RefreshCw />
              刷新
            </Button>
            <PermissionGate permissions={["schedule:job:create"]}>
              <Button onClick={() => void openEditor()}>
                <FilePlus2 />
                新增
              </Button>
            </PermissionGate>
          </div>
        </div>
      </section>

      <DataTable columns={columns} data={jobs} loading={loading} emptyText="暂无定时任务" />
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
        <DialogContent className="max-w-3xl">
          <DialogHeader>
            <DialogTitle>{editing ? "编辑定时任务" : "新增定时任务"}</DialogTitle>
            <DialogDescription>秒级 Cron、HTTP 回调或内置 Rust 任务</DialogDescription>
          </DialogHeader>
          <JobForm value={editing} submitting={submitting} onSubmit={save} onCancel={() => setOpen(false)} />
        </DialogContent>
      </Dialog>

      <Dialog open={!!deleteTarget} onOpenChange={(nextOpen) => !nextOpen && setDeleteTarget(null)}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>删除定时任务</DialogTitle>
            <DialogDescription>确认删除“{deleteTarget?.name}”？执行日志也会被删除。</DialogDescription>
          </DialogHeader>
          <div className="flex justify-end gap-2">
            <Button variant="outline" onClick={() => setDeleteTarget(null)}>
              取消
            </Button>
            <Button variant="destructive" onClick={() => void confirmRemove()}>
              删除
            </Button>
          </div>
        </DialogContent>
      </Dialog>

      <Dialog open={logsOpen} onOpenChange={setLogsOpen}>
        <DialogContent className="max-w-5xl">
          <DialogHeader>
            <DialogTitle>执行日志</DialogTitle>
            <DialogDescription>{logJob?.name}</DialogDescription>
          </DialogHeader>
          <div className="flex justify-end">
            <Button variant="outline" size="sm" onClick={() => void loadLogs()} disabled={logsLoading}>
              <RefreshCw />
              刷新
            </Button>
          </div>
          <DataTable columns={logColumns} data={logs} loading={logsLoading} emptyText="暂无执行日志" />
        </DialogContent>
      </Dialog>
    </div>
  );
}
