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
import type { DictCommand, DictItemCommand, DictItemResp, DictResp } from "@/types/system";

export function DictForm({
  value,
  submitting,
  onSubmit,
  onCancel
}: {
  value?: DictResp | null;
  submitting?: boolean;
  onSubmit: (command: DictCommand) => void;
  onCancel: () => void;
}) {
  const [form, setForm] = useState<DictCommand>({ name: "", code: "", description: "" });

  useEffect(() => {
    setForm(
      value
        ? { name: value.name, code: value.code, description: value.description }
        : { name: "", code: "", description: "" }
    );
  }, [value]);

  return (
    <form
      className="grid gap-4"
      onSubmit={(event) => {
        event.preventDefault();
        onSubmit(form);
      }}
    >
      <Field label="字典名称">
        <Input value={form.name} onChange={(event) => setForm({ ...form, name: event.target.value })} required />
      </Field>
      <Field label="字典编码">
        <Input
          value={form.code}
          onChange={(event) => setForm({ ...form, code: event.target.value })}
          disabled={!!value?.isSystem}
          required
        />
      </Field>
      <Field label="描述">
        <Textarea
          value={form.description}
          onChange={(event) => setForm({ ...form, description: event.target.value })}
        />
      </Field>
      <Actions submitting={submitting} onCancel={onCancel} />
    </form>
  );
}

export function DictItemForm({
  dictId,
  value,
  submitting,
  onSubmit,
  onCancel
}: {
  dictId: number;
  value?: DictItemResp | null;
  submitting?: boolean;
  onSubmit: (command: DictItemCommand) => void;
  onCancel: () => void;
}) {
  const [form, setForm] = useState<DictItemCommand>({
    dictId,
    label: "",
    value: "",
    color: "default",
    sort: 1,
    description: "",
    status: 1
  });

  useEffect(() => {
    setForm(
      value
        ? {
            dictId: value.dictId,
            label: value.label,
            value: value.value,
            color: value.color,
            sort: value.sort,
            description: value.description,
            status: value.status
          }
        : {
            dictId,
            label: "",
            value: "",
            color: "default",
            sort: 1,
            description: "",
            status: 1
          }
    );
  }, [dictId, value]);

  return (
    <form
      className="grid gap-4"
      onSubmit={(event) => {
        event.preventDefault();
        onSubmit(form);
      }}
    >
      <div className="grid gap-3 md:grid-cols-2">
        <Field label="标签">
          <Input value={form.label} onChange={(event) => setForm({ ...form, label: event.target.value })} required />
        </Field>
        <Field label="值">
          <Input value={form.value} onChange={(event) => setForm({ ...form, value: event.target.value })} required />
        </Field>
        <Field label="颜色">
          <Select value={form.color} onValueChange={(color) => setForm({ ...form, color })}>
            <SelectTrigger>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="default">默认</SelectItem>
              <SelectItem value="primary">主要</SelectItem>
              <SelectItem value="success">成功</SelectItem>
              <SelectItem value="warning">警告</SelectItem>
              <SelectItem value="error">错误</SelectItem>
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
            onValueChange={(status) => setForm({ ...form, status: Number(status) })}
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
      <Actions submitting={submitting} onCancel={onCancel} />
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

function Actions({ submitting, onCancel }: { submitting?: boolean; onCancel: () => void }) {
  return (
    <div className="flex justify-end gap-2">
      <Button type="button" variant="outline" onClick={onCancel}>
        取消
      </Button>
      <Button type="submit" disabled={submitting}>
        保存
      </Button>
    </div>
  );
}
