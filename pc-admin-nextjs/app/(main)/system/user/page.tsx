"use client";

import { Download, FileUp, KeyRound, Pencil, RefreshCw, ShieldCheck, Trash2, UserPlus } from "lucide-react";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { ColumnDef } from "@tanstack/react-table";
import { toast } from "sonner";
import {
  addUser,
  deleteUser,
  downloadUserImportTemplate,
  exportUser,
  getUser,
  listUser,
  parseImportUser,
  resetUserPwd,
  updateUser,
  updateUserRole
} from "@/api/system/user";
import { listRole } from "@/api/system/role";
import { listDept } from "@/api/system/dept";
import { DataTable } from "@/components/table/data-table";
import { TableActionButton, TableActions } from "@/components/table/table-actions";
import { PermissionGate } from "@/components/permission/permission-gate";
import { UserForm } from "@/components/system/user-form";
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
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue
} from "@/components/ui/select";
import { saveBlob } from "@/lib/download";
import { flattenTree } from "@/lib/tree";
import type { DeptResp, RoleResp, UserCommand, UserDetailResp, UserResp } from "@/types/system";

export default function UserPage() {
  const [users, setUsers] = useState<UserResp[]>([]);
  const [total, setTotal] = useState(0);
  const [page, setPage] = useState(1);
  const [description, setDescription] = useState("");
  const [status, setStatus] = useState("all");
  const [deptId, setDeptId] = useState("all");
  const [roles, setRoles] = useState<RoleResp[]>([]);
  const [depts, setDepts] = useState<DeptResp[]>([]);
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [editorOpen, setEditorOpen] = useState(false);
  const [editingUser, setEditingUser] = useState<UserDetailResp | null>(null);
  const [roleDialogUser, setRoleDialogUser] = useState<UserResp | null>(null);
  const [selectedRoleIds, setSelectedRoleIds] = useState<number[]>([]);
  const [passwordUser, setPasswordUser] = useState<UserResp | null>(null);
  const [newPassword, setNewPassword] = useState("");
  const fileInputRef = useRef<HTMLInputElement>(null);

  const loadMeta = useCallback(async () => {
    const [roleData, deptData] = await Promise.all([listRole(), listDept()]);
    setRoles(roleData);
    setDepts(deptData);
  }, []);

  const loadUsers = useCallback(async () => {
    setLoading(true);
    try {
      const result = await listUser({
        page,
        size: 10,
        description: description || undefined,
        status: status === "all" ? undefined : Number(status),
        deptId: deptId === "all" ? undefined : deptId,
        sort: ["id,desc"]
      });
      setUsers(result.list);
      setTotal(result.total);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "用户列表加载失败");
    } finally {
      setLoading(false);
    }
  }, [deptId, description, page, status]);

  useEffect(() => {
    void loadMeta();
  }, [loadMeta]);

  useEffect(() => {
    void loadUsers();
  }, [loadUsers]);

  const columns = useMemo<ColumnDef<UserResp>[]>(
    () => [
      { accessorKey: "username", header: "用户名" },
      { accessorKey: "nickname", header: "昵称" },
      { accessorKey: "deptName", header: "部门" },
      {
        header: "角色",
        cell: ({ row }) => row.original.roleNames.join("、") || "-"
      },
      {
        header: "状态",
        cell: ({ row }) => <StatusBadge status={row.original.status} />
      },
      { accessorKey: "createTime", header: "创建时间" },
      {
        id: "actions",
        header: "操作",
        cell: ({ row }) => (
          <TableActions>
            <PermissionGate permissions={["system:user:update"]}>
              <TableActionButton icon={Pencil} label="编辑" onClick={() => void openEditor(row.original.id)} />
            </PermissionGate>
            <PermissionGate permissions={["system:user:updateRole"]}>
              <TableActionButton icon={ShieldCheck} label="分配角色" onClick={() => openRoleDialog(row.original)} />
            </PermissionGate>
            <PermissionGate permissions={["system:user:resetPwd"]}>
              <TableActionButton icon={KeyRound} label="重置密码" onClick={() => setPasswordUser(row.original)} />
            </PermissionGate>
            <PermissionGate permissions={["system:user:delete"]}>
              <TableActionButton
                icon={Trash2}
                label="删除"
                destructive
                disabled={row.original.disabled}
                onClick={() => void removeUser(row.original.id)}
              />
            </PermissionGate>
          </TableActions>
        )
      }
    ],
    []
  );

  async function openEditor(id?: number) {
    if (!id) {
      setEditingUser(null);
      setEditorOpen(true);
      return;
    }
    try {
      setEditingUser(await getUser(id));
      setEditorOpen(true);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "用户详情加载失败");
    }
  }

  async function saveUser(command: UserCommand) {
    setSaving(true);
    try {
      if (editingUser) {
        await updateUser(editingUser.id, command);
      } else {
        await addUser(command);
      }
      setEditorOpen(false);
      await loadUsers();
      toast.success("用户已保存");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "用户保存失败");
    } finally {
      setSaving(false);
    }
  }

  async function removeUser(id: number) {
    if (!window.confirm("确认删除该用户？")) {
      return;
    }
    try {
      await deleteUser(id);
      await loadUsers();
      toast.success("用户已删除");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "用户删除失败");
    }
  }

  function openRoleDialog(user: UserResp) {
    setRoleDialogUser(user);
    setSelectedRoleIds(user.roleIds);
  }

  async function saveUserRoles() {
    if (!roleDialogUser) {
      return;
    }
    try {
      await updateUserRole(roleDialogUser.id, selectedRoleIds);
      setRoleDialogUser(null);
      await loadUsers();
      toast.success("角色已更新");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "角色更新失败");
    }
  }

  async function submitResetPassword() {
    if (!passwordUser) {
      return;
    }
    try {
      await resetUserPwd(passwordUser.id, newPassword);
      setPasswordUser(null);
      setNewPassword("");
      toast.success("密码已重置");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "密码重置失败");
    }
  }

  async function handleExport() {
    const blob = await exportUser({ description, status: status === "all" ? undefined : Number(status) });
    saveBlob(blob, "users.csv");
  }

  async function handleTemplate() {
    saveBlob(await downloadUserImportTemplate(), "user_import_template.csv");
  }

  async function handleImportFile(file: File) {
    const formData = new FormData();
    formData.append("file", file);
    try {
      await parseImportUser(formData);
      toast.success("导入文件已解析");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "导入解析失败");
    }
  }

  const deptOptions = flattenTree(depts);
  const pageCount = Math.max(1, Math.ceil(total / 10));

  return (
    <div className="mx-auto flex w-full max-w-7xl flex-col gap-4">
      <section className="flex flex-col gap-3 rounded-lg border bg-background p-4">
        <div className="flex flex-col gap-3 lg:flex-row lg:items-end lg:justify-between">
          <div className="grid gap-3 md:grid-cols-3 lg:min-w-[720px]">
            <div className="grid gap-2">
              <span className="text-sm font-medium">关键词</span>
              <Input value={description} onChange={(event) => setDescription(event.target.value)} />
            </div>
            <div className="grid gap-2">
              <span className="text-sm font-medium">状态</span>
              <Select value={status} onValueChange={setStatus}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">全部</SelectItem>
                  <SelectItem value="1">启用</SelectItem>
                  <SelectItem value="2">禁用</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="grid gap-2">
              <span className="text-sm font-medium">部门</span>
              <Select value={deptId} onValueChange={setDeptId}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">全部</SelectItem>
                  {deptOptions.map(({ node, depth }) => (
                    <SelectItem key={node.id} value={String(node.id)}>
                      {"　".repeat(depth)}
                      {node.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          </div>
          <div className="flex flex-wrap gap-2">
            <Button variant="outline" onClick={() => void loadUsers()}>
              <RefreshCw />
              刷新
            </Button>
            <PermissionGate permissions={["system:user:create"]}>
              <Button onClick={() => void openEditor()}>
                <UserPlus />
                新增
              </Button>
            </PermissionGate>
            <PermissionGate permissions={["system:user:export"]}>
              <Button variant="outline" onClick={() => void handleExport()}>
                <Download />
                导出
              </Button>
            </PermissionGate>
            <PermissionGate permissions={["system:user:import"]}>
              <Button variant="outline" onClick={() => void handleTemplate()}>
                <Download />
                模板
              </Button>
              <Button variant="outline" onClick={() => fileInputRef.current?.click()}>
                <FileUp />
                导入
              </Button>
              <input
                ref={fileInputRef}
                className="hidden"
                type="file"
                onChange={(event) => {
                  const file = event.target.files?.[0];
                  if (file) void handleImportFile(file);
                  event.target.value = "";
                }}
              />
            </PermissionGate>
          </div>
        </div>
      </section>

      <DataTable columns={columns} data={users} loading={loading} />
      <div className="flex items-center justify-end gap-2 text-sm">
        <span className="text-muted-foreground">
          第 {page} / {pageCount} 页，共 {total} 条
        </span>
        <Button variant="outline" size="sm" disabled={page <= 1} onClick={() => setPage(page - 1)}>
          上一页
        </Button>
        <Button
          variant="outline"
          size="sm"
          disabled={page >= pageCount}
          onClick={() => setPage(page + 1)}
        >
          下一页
        </Button>
      </div>

      <Dialog open={editorOpen} onOpenChange={setEditorOpen}>
        <DialogContent className="max-w-3xl">
          <DialogHeader>
            <DialogTitle>{editingUser ? "编辑用户" : "新增用户"}</DialogTitle>
            <DialogDescription>用户基础资料、部门和角色配置</DialogDescription>
          </DialogHeader>
          <UserForm
            value={editingUser}
            roles={roles}
            depts={depts}
            submitting={saving}
            onSubmit={saveUser}
            onCancel={() => setEditorOpen(false)}
          />
        </DialogContent>
      </Dialog>

      <Dialog open={!!roleDialogUser} onOpenChange={(open) => !open && setRoleDialogUser(null)}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>分配角色</DialogTitle>
            <DialogDescription>{roleDialogUser?.username}</DialogDescription>
          </DialogHeader>
          <div className="grid max-h-72 gap-2 overflow-y-auto rounded-md border p-3">
            {roles.map((role) => (
              <label key={role.id} className="flex items-center gap-2 text-sm">
                <Checkbox
                  checked={selectedRoleIds.includes(role.id)}
                  onCheckedChange={(checked) =>
                    setSelectedRoleIds(
                      checked
                        ? [...selectedRoleIds, role.id]
                        : selectedRoleIds.filter((id) => id !== role.id)
                    )
                  }
                />
                {role.name}
              </label>
            ))}
          </div>
          <div className="flex justify-end gap-2">
            <Button variant="outline" onClick={() => setRoleDialogUser(null)}>
              取消
            </Button>
            <Button onClick={() => void saveUserRoles()}>保存</Button>
          </div>
        </DialogContent>
      </Dialog>

      <Dialog open={!!passwordUser} onOpenChange={(open) => !open && setPasswordUser(null)}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>重置密码</DialogTitle>
            <DialogDescription>{passwordUser?.username}</DialogDescription>
          </DialogHeader>
          <Input
            value={newPassword}
            type="password"
            onChange={(event) => setNewPassword(event.target.value)}
            placeholder="新密码"
          />
          <div className="flex justify-end gap-2">
            <Button variant="outline" onClick={() => setPasswordUser(null)}>
              取消
            </Button>
            <Button onClick={() => void submitResetPassword()}>保存</Button>
          </div>
        </DialogContent>
      </Dialog>
    </div>
  );
}
