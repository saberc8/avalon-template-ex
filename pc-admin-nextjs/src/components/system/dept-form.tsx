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
import { Textarea } from "@/components/ui/textarea";
import type { DeptCommand, DeptResp } from "@/types/system";

interface DeptFormProps {
  value?: DeptResp | null;
  depts: DeptResp[];
  submitting?: boolean;
  onSubmit: (command: DeptCommand) => void;
  onCancel: () => void;
}

const emptyDept: DeptCommand = {
  name: "",
  parentId: 0,
  sort: 1,
  status: 1,
  description: ""
};

export function DeptForm({ value, depts, submitting, onSubmit, onCancel }: DeptFormProps) {
  const [form, setForm] = useState<DeptCommand>(emptyDept);

  useEffect(() => {
    if (!value) {
      setForm(emptyDept);
      return;
    }
    setForm({
      name: value.name,
      parentId: value.parentId,
      sort: value.sort,
      status: value.status,
      description: value.description
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
        <Field label="部门名称">
          <Input
            value={form.name}
            onChange={(event) => setForm({ ...form, name: event.target.value })}
            required
          />
        </Field>
        <Field label="上级部门">
          <Select
            value={String(form.parentId)}
            onValueChange={(nextValue) => setForm({ ...form, parentId: Number(nextValue) })}
          >
            <SelectTrigger>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="0">根部门</SelectItem>
              {flattenDeptOptions(depts).map(({ dept, depth }) => (
                <SelectItem key={dept.id} value={String(dept.id)}>
                  {"　".repeat(depth)}
                  {dept.name}
                </SelectItem>
              ))}
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
