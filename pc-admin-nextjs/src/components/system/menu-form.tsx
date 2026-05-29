"use client";

import { useEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue
} from "@/components/ui/select";
import { Switch } from "@/components/ui/switch";
import type { MenuCommand, MenuResp } from "@/types/system";

interface MenuFormProps {
  value?: MenuResp | null;
  menus: MenuResp[];
  submitting?: boolean;
  onSubmit: (command: MenuCommand) => void;
  onCancel: () => void;
}

const emptyMenu: MenuCommand = {
  type: 2,
  icon: "",
  title: "",
  sort: 1,
  permission: "",
  path: "",
  name: "",
  component: "",
  redirect: "",
  isExternal: false,
  isCache: false,
  isHidden: false,
  parentId: 0,
  status: 1
};

export function MenuForm({ value, menus, submitting, onSubmit, onCancel }: MenuFormProps) {
  const [form, setForm] = useState<MenuCommand>(emptyMenu);

  useEffect(() => {
    if (!value) {
      setForm(emptyMenu);
      return;
    }
    setForm({
      type: value.type,
      icon: value.icon,
      title: value.title,
      sort: value.sort,
      permission: value.permission,
      path: value.path,
      name: value.name,
      component: value.component,
      redirect: value.redirect,
      isExternal: value.isExternal,
      isCache: value.isCache,
      isHidden: value.isHidden,
      parentId: value.parentId,
      status: value.status
    });
  }, [value]);

  return (
    <form
      className="grid gap-4"
      onSubmit={(event) => {
        event.preventDefault();
        onSubmit(form);
      }}
    >
      <div className="grid gap-3 md:grid-cols-2">
        <Field label="菜单标题">
          <Input
            value={form.title}
            onChange={(event) => setForm({ ...form, title: event.target.value })}
            required
          />
        </Field>
        <Field label="上级菜单">
          <Select
            value={String(form.parentId)}
            onValueChange={(nextValue) => setForm({ ...form, parentId: Number(nextValue) })}
          >
            <SelectTrigger>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="0">根目录</SelectItem>
              {flattenMenuOptions(menus).map(({ menu, depth }) => (
                <SelectItem key={menu.id} value={String(menu.id)}>
                  {"　".repeat(depth)}
                  {menu.title}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </Field>
        <Field label="类型">
          <Select
            value={String(form.type)}
            onValueChange={(nextValue) => setForm({ ...form, type: Number(nextValue) as 1 | 2 | 3 })}
          >
            <SelectTrigger>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="1">目录</SelectItem>
              <SelectItem value="2">菜单</SelectItem>
              <SelectItem value="3">按钮</SelectItem>
            </SelectContent>
          </Select>
        </Field>
        <Field label="排序">
          <Input
            value={form.sort}
            type="number"
            onChange={(event) => setForm({ ...form, sort: Number(event.target.value) })}
          />
        </Field>
        <Field label="路由地址">
          <Input value={form.path} onChange={(event) => setForm({ ...form, path: event.target.value })} />
        </Field>
        <Field label="组件路径">
          <Input
            value={form.component}
            onChange={(event) => setForm({ ...form, component: event.target.value })}
          />
        </Field>
        <Field label="路由名称">
          <Input value={form.name} onChange={(event) => setForm({ ...form, name: event.target.value })} />
        </Field>
        <Field label="权限标识">
          <Input
            value={form.permission}
            onChange={(event) => setForm({ ...form, permission: event.target.value })}
          />
        </Field>
        <Field label="图标">
          <Input value={form.icon} onChange={(event) => setForm({ ...form, icon: event.target.value })} />
        </Field>
        <Field label="重定向">
          <Input
            value={form.redirect}
            onChange={(event) => setForm({ ...form, redirect: event.target.value })}
          />
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
      </div>

      <div className="grid gap-2 md:grid-cols-3">
        <Toggle label="外链" checked={form.isExternal} onChange={(isExternal) => setForm({ ...form, isExternal })} />
        <Toggle label="缓存" checked={form.isCache} onChange={(isCache) => setForm({ ...form, isCache })} />
        <Toggle label="隐藏" checked={form.isHidden} onChange={(isHidden) => setForm({ ...form, isHidden })} />
      </div>

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

function Toggle({
  label,
  checked,
  onChange
}: {
  label: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
}) {
  return (
    <label className="flex items-center justify-between rounded-md border p-3 text-sm">
      {label}
      <Switch checked={checked} onCheckedChange={onChange} />
    </label>
  );
}

function flattenMenuOptions(menus: MenuResp[], depth = 0): Array<{ menu: MenuResp; depth: number }> {
  return menus.flatMap((menu) => [
    { menu, depth },
    ...flattenMenuOptions(menu.children, depth + 1)
  ]);
}
