"use client";

import { RefreshCw, ShieldX } from "lucide-react";
import { useCallback, useEffect, useMemo, useState } from "react";
import type { ColumnDef } from "@tanstack/react-table";
import { toast } from "sonner";
import { kickout, listOnlineUser } from "@/api/monitor/online";
import { DataTable } from "@/components/table/data-table";
import { PermissionGate } from "@/components/permission/permission-gate";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import type { OnlineUserResp } from "@/types/monitor";

export default function OnlinePage() {
  const [users, setUsers] = useState<OnlineUserResp[]>([]);
  const [total, setTotal] = useState(0);
  const [page, setPage] = useState(1);
  const [nickname, setNickname] = useState("");
  const [loading, setLoading] = useState(false);

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const result = await listOnlineUser({
        page,
        size: 10,
        nickname: nickname || undefined,
        sort: ["loginTime,desc"]
      });
      setUsers(result.list);
      setTotal(result.total);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "在线用户加载失败");
    } finally {
      setLoading(false);
    }
  }, [nickname, page]);

  useEffect(() => {
    void load();
  }, [load]);

  const columns = useMemo<ColumnDef<OnlineUserResp>[]>(
    () => [
      { accessorKey: "username", header: "用户名" },
      { accessorKey: "nickname", header: "昵称" },
      { accessorKey: "clientType", header: "客户端" },
      { accessorKey: "ip", header: "IP" },
      { accessorKey: "browser", header: "浏览器" },
      { accessorKey: "os", header: "系统" },
      { accessorKey: "loginTime", header: "登录时间" },
      {
        id: "actions",
        header: "操作",
        cell: ({ row }) => (
          <PermissionGate permissions={["monitor:online:kickout"]}>
            <Button size="icon" variant="ghost" title="强退" onClick={() => void kick(row.original.token)}>
              <ShieldX />
            </Button>
          </PermissionGate>
        )
      }
    ],
    []
  );

  async function kick(token: string) {
    if (!window.confirm("确认强退该用户？")) return;
    try {
      await kickout(token);
      await load();
      toast.success("已强退");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "强退失败");
    }
  }

  const pageCount = Math.max(1, Math.ceil(total / 10));

  return (
    <div className="mx-auto flex w-full max-w-7xl flex-col gap-4">
      <section className="flex flex-col gap-3 rounded-lg border bg-background p-4 md:flex-row md:items-end md:justify-between">
        <div className="grid gap-2 md:w-80">
          <span className="text-sm font-medium">昵称</span>
          <Input value={nickname} onChange={(event) => setNickname(event.target.value)} />
        </div>
        <Button variant="outline" onClick={() => void load()}>
          <RefreshCw />
          刷新
        </Button>
      </section>
      <DataTable columns={columns} data={users} loading={loading} />
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
