"use client";

import { useEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue
} from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";
import type { DeptResp, RoleResp, UserCommand, UserDetailResp } from "@/types/system";

interface UserFormProps {
  value?: UserDetailResp | null;
  roles: RoleResp[];
  depts: DeptResp[];
  submitting?: boolean;
  onSubmit: (command: UserCommand) => void;
  onCancel: () => void;
}

const emptyUser: UserCommand = {
  username: "",
  nickname: "",
  password: "",
  gender: 0,
  email: "",
  phone: "",
  avatar: "",
  description: "",
  status: 1,
  deptId: 0,
  roleIds: []
};

export function UserForm({ value, roles, depts, submitting, onSubmit, onCancel }: UserFormProps) {
  const [form, setForm] = useState<UserCommand>(emptyUser);

  useEffect(() => {
    if (!value) {
      setForm({ ...emptyUser, deptId: flattenDeptOptions(depts)[0]?.dept.id ?? 0 });
      return;
    }
    setForm({
      username: value.username,
      nickname: value.nickname,
      password: "",
      gender: value.gender,
      email: value.email,
      phone: value.phone,
      avatar: value.avatar,
      description: value.description,
      status: value.status,
      deptId: value.deptId,
      roleIds: value.roleIds
    });
  }, [depts, value]);

  return (
    <form
      className="grid gap-4"
      onSubmit={(event) => {
        event.preventDefault();
        onSubmit(form);
      }}
    >
      <div className="grid gap-3 md:grid-cols-2">
        <Field label="用户名">
          <Input
            value={form.username}
            onChange={(event) => setForm({ ...form, username: event.target.value })}
            required
            disabled={!!value}
          />
        </Field>
        <Field label="昵称">
          <Input
            value={form.nickname}
            onChange={(event) => setForm({ ...form, nickname: event.target.value })}
            required
          />
        </Field>
        {!value ? (
          <Field label="初始密码">
            <Input
              value={form.password}
              type="password"
              onChange={(event) => setForm({ ...form, password: event.target.value })}
              required
            />
          </Field>
        ) : null}
        <Field label="部门">
          <Select
            value={String(form.deptId)}
            onValueChange={(nextValue) => setForm({ ...form, deptId: Number(nextValue) })}
          >
            <SelectTrigger>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="0">未分配</SelectItem>
              {flattenDeptOptions(depts).map(({ dept, depth }) => (
                <SelectItem key={dept.id} value={String(dept.id)}>
                  {"　".repeat(depth)}
                  {dept.name}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </Field>
        <Field label="性别">
          <Select
            value={String(form.gender)}
            onValueChange={(nextValue) => setForm({ ...form, gender: Number(nextValue) })}
          >
            <SelectTrigger>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="0">未知</SelectItem>
              <SelectItem value="1">男</SelectItem>
              <SelectItem value="2">女</SelectItem>
            </SelectContent>
          </Select>
        </Field>
        <Field label="状态">
          <Select
            value={String(form.status)}
            onValueChange={(nextValue) => setForm({ ...form, status: Number(nextValue) })}
          >
            <SelectTrigger>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="1">启用</SelectItem>
              <SelectItem value="2">禁用</SelectItem>
            </SelectContent>
          </Select>
        </Field>
        <Field label="邮箱">
          <Input
            value={form.email}
            type="email"
            onChange={(event) => setForm({ ...form, email: event.target.value })}
          />
        </Field>
        <Field label="手机号">
          <Input
            value={form.phone}
            onChange={(event) => setForm({ ...form, phone: event.target.value })}
          />
        </Field>
      </div>

      <Field label="角色">
        <div className="grid max-h-32 gap-2 overflow-y-auto rounded-md border p-3 md:grid-cols-2">
          {roles.map((role) => (
            <label key={role.id} className="flex items-center gap-2 text-sm">
              <Checkbox
                checked={form.roleIds.includes(role.id)}
                onCheckedChange={(checked) =>
                  setForm({
                    ...form,
                    roleIds: checked
                      ? [...form.roleIds, role.id]
                      : form.roleIds.filter((id) => id !== role.id)
                  })
                }
              />
              {role.name}
            </label>
          ))}
        </div>
      </Field>

      <Field label="描述">
        <Textarea
          value={form.description}
          onChange={(event) => setForm({ ...form, description: event.target.value })}
        />
      </Field>

      <div className="flex justify-end gap-2">
        <Button type="button" variant="outline" onClick={onCancel}>
          取消
        </Button>
        <Button type="submit" disabled={submitting}>
          保存
        </Button>
      </div>
    </form>
  );
}

function Field({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="grid gap-2">
      <Label>{label}</Label>
      {children}
    </div>
  );
}

function flattenDeptOptions(depts: DeptResp[], depth = 0): Array<{ dept: DeptResp; depth: number }> {
  return depts.flatMap((dept) => [
    { dept, depth },
    ...flattenDeptOptions(dept.children, depth + 1)
  ]);
}
