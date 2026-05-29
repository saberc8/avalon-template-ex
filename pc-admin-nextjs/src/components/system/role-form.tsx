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
import { Switch } from "@/components/ui/switch";
import { Textarea } from "@/components/ui/textarea";
import type { DeptResp, RoleCommand, RoleDetailResp } from "@/types/system";

interface RoleFormProps {
  value?: RoleDetailResp | null;
  depts: DeptResp[];
  submitting?: boolean;
  onSubmit: (command: RoleCommand) => void;
  onCancel: () => void;
}

const emptyRole: RoleCommand = {
  name: "",
  code: "",
  sort: 1,
  description: "",
  dataScope: 4,
  deptIds: [],
  deptCheckStrictly: true
};

export function RoleForm({ value, depts, submitting, onSubmit, onCancel }: RoleFormProps) {
  const [form, setForm] = useState<RoleCommand>(emptyRole);

  useEffect(() => {
    if (!value) {
      setForm(emptyRole);
      return;
    }
    setForm({
      name: value.name,
      code: value.code,
      sort: value.sort,
      description: value.description,
      dataScope: value.dataScope,
      deptIds: value.deptIds,
      deptCheckStrictly: value.deptCheckStrictly
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
        <Field label="角色名称">
          <Input
            value={form.name}
            onChange={(event) => setForm({ ...form, name: event.target.value })}
            required
          />
        </Field>
        <Field label="角色编码">
          <Input
            value={form.code}
            onChange={(event) => setForm({ ...form, code: event.target.value })}
            required
            disabled={!!value?.isSystem}
          />
        </Field>
        <Field label="排序">
          <Input
            value={form.sort}
            type="number"
            onChange={(event) => setForm({ ...form, sort: Number(event.target.value) })}
          />
        </Field>
        <Field label="数据权限">
          <Select
            value={String(form.dataScope)}
            onValueChange={(nextValue) => setForm({ ...form, dataScope: Number(nextValue) })}
          >
            <SelectTrigger>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="1">全部数据</SelectItem>
              <SelectItem value="2">本部门及以下</SelectItem>
              <SelectItem value="3">本部门</SelectItem>
              <SelectItem value="4">仅本人</SelectItem>
              <SelectItem value="5">自定义部门</SelectItem>
            </SelectContent>
          </Select>
        </Field>
      </div>

      {form.dataScope === 5 ? (
        <Field label="自定义部门">
          <div className="grid max-h-36 gap-2 overflow-y-auto rounded-md border p-3 md:grid-cols-2">
            {flattenDeptOptions(depts).map(({ dept, depth }) => (
              <label key={dept.id} className="flex items-center gap-2 text-sm">
                <Checkbox
                  checked={form.deptIds.includes(dept.id)}
                  onCheckedChange={(checked) =>
                    setForm({
                      ...form,
                      deptIds: checked
                        ? [...form.deptIds, dept.id]
                        : form.deptIds.filter((id) => id !== dept.id)
                    })
                  }
                />
                <span>
                  {"　".repeat(depth)}
                  {dept.name}
                </span>
              </label>
            ))}
          </div>
        </Field>
      ) : null}

      <label className="flex items-center justify-between rounded-md border p-3 text-sm">
        部门父子联动
        <Switch
          checked={form.deptCheckStrictly}
          onCheckedChange={(checked) => setForm({ ...form, deptCheckStrictly: checked })}
        />
      </label>

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
