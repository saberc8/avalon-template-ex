"use client";

import { RefreshCw, RotateCcw, Save } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { toast } from "sonner";
import { listOption, resetOptionValue, updateOption } from "@/api/system/option";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import type { OptionResp } from "@/types/system";

export function ConfigOptionPanel({ category }: { category: string }) {
  const [options, setOptions] = useState<OptionResp[]>([]);
  const [values, setValues] = useState<Record<number, string>>({});
  const [loading, setLoading] = useState(false);

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
      await loadOptions();
      toast.success("配置已重置");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "配置重置失败");
    }
  }

  return (
    <section className="grid gap-4">
      <div className="flex flex-wrap justify-end gap-2">
        <Button variant="outline" onClick={() => void loadOptions()} disabled={loading}>
          <RefreshCw />
          刷新
        </Button>
        <Button variant="outline" onClick={() => void resetOptions()}>
          <RotateCcw />
          重置
        </Button>
        <Button onClick={() => void saveOptions()}>
          <Save />
          保存
        </Button>
      </div>
      <div className="grid gap-3 md:grid-cols-2">
        {options.map((option) => (
          <div key={option.id} className="grid gap-2 rounded-lg border bg-background p-3">
            <Label>{option.name}</Label>
            <Input
              value={values[option.id] ?? ""}
              onChange={(event) => setValues({ ...values, [option.id]: event.target.value })}
            />
            <div className="text-xs text-muted-foreground">{option.code}</div>
          </div>
        ))}
      </div>
    </section>
  );
}
