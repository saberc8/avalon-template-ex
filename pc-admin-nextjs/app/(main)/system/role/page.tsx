"use client";

import { Pencil, RefreshCw, ShieldCheck, Trash2, UserCheck, UserPlus } from "lucide-react";
import { useCallback, useEffect, useMemo, useState } from "react";
import type { ColumnDef } from "@tanstack/react-table";
import { toast } from "sonner";
import {
  addRole,
  assignToUsers,
  deleteRole,
  getRole,
  listRole,
  listRoleUser,
  listRoleUserId,
  unassignFromUsers,
  updateRole,
  updateRolePermission
} from "@/api/system/role";
import { listDept } from "@/api/system/dept";
import { listMenu } from "@/api/system/menu";
import { listAllUser } from "@/api/system/user";
import { DataTable } from "@/components/table/data-table";
import { TableActionButton, TableActions } from "@/components/table/table-actions";
import { PermissionGate } from "@/components/permission/permission-gate";
import { PermissionTree } from "@/components/system/permission-tree";
import { RoleForm } from "@/components/system/role-form";
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
import { Switch } from "@/components/ui/switch";
import type {
  DeptResp,
  MenuResp,
  RoleCommand,
  RoleDetailResp,
  RoleResp,
  RoleUserResp,
  UserResp
} from "@/types/system";

const dataScopeLabels: Record<number, string> = {
  1: "全部数据",
  2: "本部门及以下",
  3: "本部门",
  4: "仅本人",
  5: "自定义部门"
};

export default function RolePage() {
  const [roles, setRoles] = useState<RoleResp[]>([]);
  const [depts, setDepts] = useState<DeptResp[]>([]);
  const [menus, setMenus] = useState<MenuResp[]>([]);
  const [users, setUsers] = useState<UserResp[]>([]);
  const [description, setDescription] = useState("");
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [editorOpen, setEditorOpen] = useState(false);
  const [editingRole, setEditingRole] = useState<RoleDetailResp | null>(null);
  const [permissionRole, setPermissionRole] = useState<RoleDetailResp | null>(null);
  const [selectedMenuIds, setSelectedMenuIds] = useState<number[]>([]);
  const [menuCheckStrictly, setMenuCheckStrictly] = useState(true);
  const [assignedRole, setAssignedRole] = useState<RoleResp | null>(null);
  const [assignedUsers, setAssignedUsers] = useState<RoleUserResp[]>([]);
  const [selectedUserIds, setSelectedUserIds] = useState<number[]>([]);

  const loadMeta = useCallback(async () => {
    const [deptData, menuData, userData] = await Promise.all([
      listDept(),
      listMenu(),
      listAllUser({ sort: ["id,desc"] })
    ]);
    setDepts(deptData);
    setMenus(menuData);
    setUsers(userData);
  }, []);

  const loadRoles = useCallback(async () => {
    setLoading(true);
    try {
      setRoles(await listRole({ description: description || undefined, sort: ["sort,asc"] }));
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "角色列表加载失败");
    } finally {
      setLoading(false);
    }
  }, [description]);

  useEffect(() => {
    void loadMeta();
  }, [loadMeta]);

  useEffect(() => {
    void loadRoles();
  }, [loadRoles]);

  const columns = useMemo<ColumnDef<RoleResp>[]>(
    () => [
      { accessorKey: "name", header: "角色名称" },
      { accessorKey: "code", header: "角色编码" },
      {
        header: "数据权限",
        cell: ({ row }) => dataScopeLabels[row.original.dataScope] ?? row.original.dataScope
      },
      { accessorKey: "sort", header: "排序" },
      { accessorKey: "createTime", header: "创建时间" },
      {
        id: "actions",
        header: "操作",
        cell: ({ row }) => (
          <TableActions>
            <PermissionGate permissions={["system:role:update"]}>
              <TableActionButton icon={Pencil} label="编辑" onClick={() => void openEditor(row.original.id)} />
            </PermissionGate>
            <PermissionGate permissions={["system:role:updatePermission"]}>
              <TableActionButton
                icon={ShieldCheck}
                label="权限"
                onClick={() => void openPermissionDialog(row.original.id)}
              />
            </PermissionGate>
            <PermissionGate permissions={["system:role:assign"]}>
              <TableActionButton icon={UserCheck} label="分配用户" onClick={() => void openAssignedUsers(row.original)} />
            </PermissionGate>
            <PermissionGate permissions={["system:role:delete"]}>
              <TableActionButton
                icon={Trash2}
                label="删除"
                destructive
                disabled={row.original.disabled}
                onClick={() => void removeRole(row.original.id)}
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
      setEditingRole(null);
      setEditorOpen(true);
      return;
    }
    try {
      setEditingRole(await getRole(id));
      setEditorOpen(true);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "角色详情加载失败");
    }
  }

  async function saveRole(command: RoleCommand) {
    setSaving(true);
    try {
      if (editingRole) {
        await updateRole(editingRole.id, command);
      } else {
        await addRole(command);
      }
      setEditorOpen(false);
      await loadRoles();
      toast.success("角色已保存");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "角色保存失败");
    } finally {
      setSaving(false);
    }
  }

  async function removeRole(id: number) {
    if (!window.confirm("确认删除该角色？")) {
      return;
    }
    try {
      await deleteRole(id);
      await loadRoles();
      toast.success("角色已删除");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "角色删除失败");
    }
  }

  async function openPermissionDialog(id: number) {
    try {
      const detail = await getRole(id);
      setPermissionRole(detail);
      setSelectedMenuIds(detail.menuIds);
      setMenuCheckStrictly(detail.menuCheckStrictly);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "角色权限加载失败");
    }
  }

  async function savePermission() {
    if (!permissionRole) {
      return;
    }
    try {
      await updateRolePermission(permissionRole.id, { menuIds: selectedMenuIds, menuCheckStrictly });
      setPermissionRole(null);
      toast.success("角色权限已更新");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "角色权限更新失败");
    }
  }

  async function openAssignedUsers(role: RoleResp) {
    setAssignedRole(role);
    try {
      const [pageResult, userIds] = await Promise.all([
        listRoleUser(role.id, { page: 1, size: 50, sort: ["id,desc"] }),
        listRoleUserId(role.id)
      ]);
      setAssignedUsers(pageResult.list);
      setSelectedUserIds(userIds);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "关联用户加载失败");
    }
  }

  async function saveAssignedUsers() {
    if (!assignedRole) {
      return;
    }
    try {
      await assignToUsers(assignedRole.id, selectedUserIds);
      await openAssignedUsers(assignedRole);
      toast.success("用户分配已更新");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "用户分配失败");
    }
  }

  async function unassignUser(userRoleId: number) {
    try {
      await unassignFromUsers([userRoleId]);
      if (assignedRole) {
        await openAssignedUsers(assignedRole);
      }
      toast.success("已取消分配");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "取消分配失败");
    }
  }

  return (
    <div className="mx-auto flex w-full max-w-7xl flex-col gap-4">
      <section className="flex flex-col gap-3 rounded-lg border bg-background p-4 md:flex-row md:items-end md:justify-between">
        <div className="grid gap-2 md:w-80">
          <span className="text-sm font-medium">关键词</span>
          <Input value={description} onChange={(event) => setDescription(event.target.value)} />
        </div>
        <div className="flex flex-wrap gap-2">
          <Button variant="outline" onClick={() => void loadRoles()}>
            <RefreshCw />
            刷新
          </Button>
          <PermissionGate permissions={["system:role:create"]}>
            <Button onClick={() => void openEditor()}>
              <UserPlus />
              新增
            </Button>
          </PermissionGate>
        </div>
      </section>

      <DataTable columns={columns} data={roles} loading={loading} />

      <Dialog open={editorOpen} onOpenChange={setEditorOpen}>
        <DialogContent className="max-w-3xl">
          <DialogHeader>
            <DialogTitle>{editingRole ? "编辑角色" : "新增角色"}</DialogTitle>
            <DialogDescription>角色基础资料和数据权限范围</DialogDescription>
          </DialogHeader>
          <RoleForm
            value={editingRole}
            depts={depts}
            submitting={saving}
            onSubmit={saveRole}
            onCancel={() => setEditorOpen(false)}
          />
        </DialogContent>
      </Dialog>

      <Dialog open={!!permissionRole} onOpenChange={(open) => !open && setPermissionRole(null)}>
        <DialogContent className="max-w-3xl">
          <DialogHeader>
            <DialogTitle>权限配置</DialogTitle>
            <DialogDescription>{permissionRole?.name}</DialogDescription>
          </DialogHeader>
          <label className="flex items-center justify-between rounded-md border p-3 text-sm">
            菜单父子联动
            <Switch checked={menuCheckStrictly} onCheckedChange={setMenuCheckStrictly} />
          </label>
          <PermissionTree menus={menus} selectedIds={selectedMenuIds} onChange={setSelectedMenuIds} />
          <div className="flex justify-end gap-2">
            <Button variant="outline" onClick={() => setPermissionRole(null)}>
              取消
            </Button>
            <Button onClick={() => void savePermission()}>保存</Button>
          </div>
        </DialogContent>
      </Dialog>

      <Dialog open={!!assignedRole} onOpenChange={(open) => !open && setAssignedRole(null)}>
        <DialogContent className="max-w-4xl">
          <DialogHeader>
            <DialogTitle>分配用户</DialogTitle>
            <DialogDescription>{assignedRole?.name}</DialogDescription>
          </DialogHeader>
          <div className="grid gap-4 md:grid-cols-[1fr_1fr]">
            <div>
              <div className="mb-2 text-sm font-medium">可选用户</div>
              <div className="max-h-80 overflow-y-auto rounded-md border p-3">
                {users.map((user) => (
                  <label key={user.id} className="flex items-center gap-2 py-1 text-sm">
                    <Checkbox
                      checked={selectedUserIds.includes(user.id)}
                      onCheckedChange={(checked) =>
                        setSelectedUserIds(
                          checked
                            ? [...selectedUserIds, user.id]
                            : selectedUserIds.filter((id) => id !== user.id)
                        )
                      }
                    />
                    {user.username} · {user.nickname}
                  </label>
                ))}
              </div>
              <Button className="mt-3" onClick={() => void saveAssignedUsers()}>
                保存分配
              </Button>
            </div>
            <div>
              <div className="mb-2 text-sm font-medium">已分配用户</div>
              <div className="max-h-80 overflow-y-auto rounded-md border">
                {assignedUsers.map((user) => (
                  <div key={user.id} className="flex items-center justify-between border-b px-3 py-2 text-sm last:border-b-0">
                    <span>
                      {user.username} · {user.nickname}
                    </span>
                    <PermissionGate permissions={["system:role:unassign"]}>
                      <Button size="sm" variant="ghost" onClick={() => void unassignUser(user.id)}>
                        取消
                      </Button>
                    </PermissionGate>
                  </div>
                ))}
              </div>
            </div>
          </div>
        </DialogContent>
      </Dialog>
    </div>
  );
}
