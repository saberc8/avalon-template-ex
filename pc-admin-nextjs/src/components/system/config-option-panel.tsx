"use client";

import { RefreshCw, RotateCcw, Save } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { toast } from "sonner";
import { listOption, resetOptionValue, updateOption } from "@/api/system/option";
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
import { Label } from "@/components/ui/label";
import type { OptionResp } from "@/types/system";

const categoryMeta: Record<string, { title: string; description: string }> = {
  SITE: {
    title: "网站配置",
    description: "站点名称、备案信息和前台展示参数"
  },
  PASSWORD: {
    title: "安全配置",
    description: "密码复杂度、登录限制和账号安全策略"
  },
  LOGIN: {
    title: "登录配置",
    description: "登录页面、认证方式和会话参数"
  }
};

export function ConfigOptionPanel({ category }: { category: string }) {
  const [options, setOptions] = useState<OptionResp[]>([]);
  const [values, setValues] = useState<Record<number, string>>({});
  const [loading, setLoading] = useState(false);
  const [resetOpen, setResetOpen] = useState(false);

  const loadOptions = useCallback(async () => {
    setLoading(true);
    try {
      const data = await listOption({ category });
      setOptions(data);
      setValues(Object.fromEntries(data.map((option) => [option.id, option.value])));
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "配置加载失败");
    } finally {
      setLoading(false);
    }
  }, [category]);

  useEffect(() => {
    void loadOptions();
  }, [loadOptions]);

  async function saveOptions() {
    try {
      await updateOption(
        options.map((option) => ({
          id: option.id,
          code: option.code,
          value: values[option.id] ?? ""
        }))
      );
      await loadOptions();
      toast.success("配置已保存");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "配置保存失败");
    }
  }

  async function resetOptions() {
    try {
      await resetOptionValue({ category });
      setResetOpen(false);
      await loadOptions();
      toast.success("配置已重置");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "配置重置失败");
    }
  }

  const meta = categoryMeta[category] ?? { title: "系统配置", description: "系统运行参数" };
  const dirtyCount = options.filter((option) => (values[option.id] ?? "") !== option.value).length;

  return (
    <section className="grid gap-4">
      <div className="flex flex-col gap-3 rounded-lg border bg-background p-4 md:flex-row md:items-center md:justify-between">
        <div>
          <div className="flex items-center gap-2">
            <h2 className="text-base font-semibold">{meta.title}</h2>
            {dirtyCount ? <Badge variant="secondary">{dirtyCount} 项未保存</Badge> : null}
          </div>
          <p className="mt-1 text-sm text-muted-foreground">{meta.description}</p>
        </div>
        <div className="flex flex-wrap gap-2">
          <Button variant="outline" onClick={() => void loadOptions()} disabled={loading}>
            <RefreshCw />
            刷新
          </Button>
          <Button variant="outline" onClick={() => setResetOpen(true)} disabled={loading || !options.length}>
            <RotateCcw />
            重置
          </Button>
          <Button onClick={() => void saveOptions()} disabled={loading || dirtyCount === 0}>
            <Save />
            保存
          </Button>
        </div>
      </div>
      <div className="grid gap-3 md:grid-cols-2">
        {loading ? <div className="col-span-full rounded-lg border border-dashed p-8 text-center text-sm text-muted-foreground">加载中</div> : null}
        {!loading && !options.length ? (
          <div className="col-span-full rounded-lg border border-dashed p-8 text-center text-sm text-muted-foreground">暂无配置项</div>
        ) : null}
        {options.map((option) => (
          <div key={option.id} className="grid gap-2 rounded-lg border bg-background p-3">
            <div className="flex items-start justify-between gap-3">
              <Label>{option.name}</Label>
              {(values[option.id] ?? "") !== option.value ? <Badge variant="outline">已修改</Badge> : null}
            </div>
            <Input
              value={values[option.id] ?? ""}
              placeholder={option.description || option.name}
              onChange={(event) => setValues({ ...values, [option.id]: event.target.value })}
            />
            <div className="flex flex-wrap gap-x-3 gap-y-1 text-xs text-muted-foreground">
              <span>{option.code}</span>
              {option.description ? <span>{option.description}</span> : null}
            </div>
          </div>
        ))}
      </div>
      <Dialog open={resetOpen} onOpenChange={setResetOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>重置{meta.title}</DialogTitle>
            <DialogDescription>将当前分组恢复为默认值，未保存的修改也会丢失。</DialogDescription>
          </DialogHeader>
          <div className="flex justify-end gap-2">
            <Button variant="outline" onClick={() => setResetOpen(false)}>
              取消
            </Button>
            <Button variant="destructive" onClick={() => void resetOptions()}>
              确认重置
            </Button>
          </div>
        </DialogContent>
      </Dialog>
    </section>
  );
}
