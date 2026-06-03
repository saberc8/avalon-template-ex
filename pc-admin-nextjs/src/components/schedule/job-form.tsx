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
import { Textarea } from "@/components/ui/textarea";
import type { JobCommand, JobResp } from "@/types/schedule";

const emptyJob: JobCommand = {
  name: "",
  groupName: "default",
  taskType: 1,
  cronExpression: "0 */5 * * * *",
  status: 2,
  concurrent: false,
  misfirePolicy: 1,
  maxRetry: 0,
  timeoutSeconds: 30,
  httpMethod: "POST",
  httpUrl: "http://localhost:4398/health",
  httpHeaders: {},
  httpBody: "",
  builtinKey: "",
  description: ""
};

export function JobForm({
  value,
  submitting,
  onSubmit,
  onCancel
}: {
  value?: JobResp | null;
  submitting?: boolean;
  onSubmit: (command: JobCommand) => void;
  onCancel: () => void;
}) {
  const [form, setForm] = useState<JobCommand>(emptyJob);
  const [headersText, setHeadersText] = useState("{}");
  const [headersError, setHeadersError] = useState("");

  useEffect(() => {
    const next = value
      ? {
          name: value.name,
          groupName: value.groupName,
          taskType: value.taskType,
          cronExpression: value.cronExpression,
          status: value.status,
          concurrent: value.concurrent,
          misfirePolicy: value.misfirePolicy,
          maxRetry: value.maxRetry,
          timeoutSeconds: value.timeoutSeconds,
          httpMethod: value.httpMethod || "POST",
          httpUrl: value.httpUrl,
          httpHeaders: value.httpHeaders ?? {},
          httpBody: value.httpBody,
          builtinKey: value.builtinKey,
          description: value.description
        }
      : { ...emptyJob };
    setForm(next);
    setHeadersText(JSON.stringify(next.httpHeaders ?? {}, null, 2));
    setHeadersError("");
  }, [value]);

  function submit() {
    let headers: Record<string, unknown> = {};
    try {
      const parsed = JSON.parse(headersText || "{}") as unknown;
      if (parsed && typeof parsed === "object" && !Array.isArray(parsed)) {
        headers = parsed as Record<string, unknown>;
      } else {
        setHeadersError("Headers 必须是 JSON 对象");
        return;
      }
    } catch {
      setHeadersError("Headers JSON 格式不正确");
      return;
    }
    setHeadersError("");
    onSubmit({ ...form, httpHeaders: headers });
  }

  return (
    <form
      className="grid gap-4"
      onSubmit={(event) => {
        event.preventDefault();
        submit();
      }}
    >
      <div className="grid gap-3 md:grid-cols-2">
        <Field label="任务名称">
          <Input value={form.name} onChange={(event) => setForm({ ...form, name: event.target.value })} required />
        </Field>
        <Field label="任务分组">
          <Input
            value={form.groupName}
            onChange={(event) => setForm({ ...form, groupName: event.target.value })}
          />
        </Field>
        <Field label="任务类型">
          <Select
            value={String(form.taskType)}
            onValueChange={(taskType) => setForm({ ...form, taskType: Number(taskType) })}
          >
            <SelectTrigger>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="1">HTTP 回调</SelectItem>
              <SelectItem value="2">内置任务</SelectItem>
            </SelectContent>
          </Select>
        </Field>
        <Field label="Cron 表达式">
          <Input
            value={form.cronExpression}
            onChange={(event) => setForm({ ...form, cronExpression: event.target.value })}
            required
          />
        </Field>
        <Field label="状态">
          <Select value={String(form.status)} onValueChange={(status) => setForm({ ...form, status: Number(status) })}>
            <SelectTrigger>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="1">启用</SelectItem>
              <SelectItem value="2">禁用</SelectItem>
            </SelectContent>
          </Select>
        </Field>
        <Field label="错过策略">
          <Select
            value={String(form.misfirePolicy)}
            onValueChange={(misfirePolicy) => setForm({ ...form, misfirePolicy: Number(misfirePolicy) })}
          >
            <SelectTrigger>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="1">立即补偿一次</SelectItem>
              <SelectItem value="2">跳过</SelectItem>
            </SelectContent>
          </Select>
        </Field>
        <Field label="最大重试次数">
          <Input
            type="number"
            min={0}
            max={10}
            value={form.maxRetry}
            onChange={(event) => setForm({ ...form, maxRetry: Number(event.target.value) })}
          />
        </Field>
        <Field label="超时时间（秒）">
          <Input
            type="number"
            min={1}
            max={3600}
            value={form.timeoutSeconds}
            onChange={(event) => setForm({ ...form, timeoutSeconds: Number(event.target.value) })}
          />
        </Field>
      </div>

      <label className="flex items-center justify-between rounded-md border p-3 text-sm">
        <span>允许并发执行</span>
        <Switch checked={form.concurrent} onCheckedChange={(concurrent) => setForm({ ...form, concurrent })} />
      </label>

      {form.taskType === 1 ? (
        <div className="grid gap-3 md:grid-cols-[140px_1fr]">
          <Field label="HTTP 方法">
            <Select value={form.httpMethod} onValueChange={(httpMethod) => setForm({ ...form, httpMethod })}>
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {["GET", "POST", "PUT", "PATCH", "DELETE"].map((method) => (
                  <SelectItem key={method} value={method}>
                    {method}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </Field>
          <Field label="HTTP URL">
            <Input
              value={form.httpUrl}
              onChange={(event) => setForm({ ...form, httpUrl: event.target.value })}
              required={form.taskType === 1}
            />
          </Field>
          <div className="md:col-span-2">
            <Field label="Headers JSON">
              <Textarea value={headersText} onChange={(event) => setHeadersText(event.target.value)} />
              {headersError ? <span className="text-xs text-destructive">{headersError}</span> : null}
            </Field>
          </div>
          <div className="md:col-span-2">
            <Field label="请求体">
              <Textarea value={form.httpBody} onChange={(event) => setForm({ ...form, httpBody: event.target.value })} />
            </Field>
          </div>
        </div>
      ) : (
        <Field label="内置任务标识">
          <Input
            value={form.builtinKey}
            placeholder="system.noop"
            onChange={(event) => setForm({ ...form, builtinKey: event.target.value })}
            required={form.taskType === 2}
          />
        </Field>
      )}

      <Field label="描述">
        <Textarea value={form.description} onChange={(event) => setForm({ ...form, description: event.target.value })} />
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
