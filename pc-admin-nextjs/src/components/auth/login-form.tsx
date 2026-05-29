"use client";

import { zodResolver } from "@hookform/resolvers/zod";
import { Activity, Layers3, LockKeyhole, LogIn, ShieldCheck, User } from "lucide-react";
import { useRouter } from "next/navigation";
import { useEffect, useState } from "react";
import { useForm } from "react-hook-form";
import { toast } from "sonner";
import { z } from "zod";
import { accountLogin, getUserInfo, getUserRoutes } from "@/api/auth";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage
} from "@/components/ui/form";
import { Input } from "@/components/ui/input";
import { getToken, setToken } from "@/lib/auth";
import { firstAccessiblePath } from "@/lib/menu";
import { cn } from "@/lib/utils";

const loginSchema = z.object({
  username: z.string().min(1, "请输入用户名"),
  password: z.string().min(1, "请输入密码"),
  captcha: z.string().optional()
});

type LoginValues = z.infer<typeof loginSchema>;

export function LoginForm({ className }: { className?: string }) {
  const router = useRouter();
  const [submitting, setSubmitting] = useState(false);
  const form = useForm<LoginValues>({
    resolver: zodResolver(loginSchema),
    defaultValues: {
      username: "admin",
      password: "admin123",
      captcha: "local"
    }
  });

  useEffect(() => {
    if (getToken()) {
      router.replace("/dashboard/workplace");
    }
  }, [router]);

  async function onSubmit(values: LoginValues) {
    setSubmitting(true);
    try {
      const loginResult = await accountLogin({
        ...values,
        uuid: values.captcha ? "local" : undefined
      });
      setToken(loginResult.token);

      const [, routes] = await Promise.all([getUserInfo(), getUserRoutes()]);
      router.replace(firstAccessiblePath(routes));
      toast.success("登录成功");
    } catch (error) {
      const message = error instanceof Error ? error.message : "登录失败";
      form.setError("root", { message });
      toast.error(message);
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <div className={cn("flex flex-col gap-6", className)}>
      <Card className="overflow-hidden border bg-background shadow-sm">
        <CardContent className="grid p-0 md:grid-cols-[1fr_1.05fr]">
          <Form {...form}>
            <form className="p-6 md:p-8" onSubmit={form.handleSubmit(onSubmit)}>
              <div className="flex flex-col gap-6">
                <div className="space-y-2 text-center">
                  <div className="mx-auto flex size-10 items-center justify-center rounded-lg bg-primary text-primary-foreground">
                    <Layers3 className="size-5" />
                  </div>
                  <div>
                    <h1 className="text-2xl font-semibold">Avalon Admin</h1>
                    <p className="mt-1 text-sm text-muted-foreground">登录后台管理系统</p>
                  </div>
                </div>
                <FormField
                  control={form.control}
                  name="username"
                  render={({ field }) => (
                    <FormItem>
                      <FormLabel>用户名</FormLabel>
                      <FormControl>
                        <div className="relative">
                          <User className="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
                          <Input className="pl-9" autoComplete="username" {...field} />
                        </div>
                      </FormControl>
                      <FormMessage />
                    </FormItem>
                  )}
                />
                <FormField
                  control={form.control}
                  name="password"
                  render={({ field }) => (
                    <FormItem>
                      <FormLabel>密码</FormLabel>
                      <FormControl>
                        <div className="relative">
                          <LockKeyhole className="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
                          <Input className="pl-9" type="password" autoComplete="current-password" {...field} />
                        </div>
                      </FormControl>
                      <FormMessage />
                    </FormItem>
                  )}
                />
                <FormField
                  control={form.control}
                  name="captcha"
                  render={({ field }) => (
                    <FormItem>
                      <FormLabel>验证码</FormLabel>
                      <FormControl>
                        <div className="relative">
                          <ShieldCheck className="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
                          <Input className="pl-9" autoComplete="one-time-code" {...field} />
                        </div>
                      </FormControl>
                      <FormMessage />
                    </FormItem>
                  )}
                />
                {form.formState.errors.root?.message ? (
                  <p className="rounded-md border border-destructive/25 bg-destructive/10 px-3 py-2 text-sm text-destructive">
                    {form.formState.errors.root.message}
                  </p>
                ) : null}
                <Button className="w-full" type="submit" disabled={submitting}>
                  <LogIn />
                  {submitting ? "登录中" : "登录"}
                </Button>
              </div>
            </form>
          </Form>
          <div className="relative hidden overflow-hidden border-l bg-sidebar md:block">
            <div className="absolute inset-0 bg-[linear-gradient(to_right,hsl(var(--sidebar-border))_1px,transparent_1px),linear-gradient(to_bottom,hsl(var(--sidebar-border))_1px,transparent_1px)] bg-[size:28px_28px] opacity-35" />
            <div className="relative flex h-full flex-col justify-between p-8 text-sidebar-foreground">
              <div>
                <div className="inline-flex items-center gap-2 rounded-full border bg-background/70 px-3 py-1 text-xs text-muted-foreground shadow-sm">
                  <Activity className="size-3.5 text-primary" />
                  DDD Rust API + Next.js Console
                </div>
                <h2 className="mt-6 max-w-sm text-2xl font-semibold leading-tight">
                  面向权限、数据和运营流程的后台管理基座
                </h2>
              </div>
              <div className="grid">
                {["数据级权限", "动态路由菜单", "用户角色授权"].map((item, index) => (
                  <div key={item} className="border-t border-sidebar-border/80 py-3">
                    <div className="text-xs text-muted-foreground">0{index + 1}</div>
                    <div className="mt-2 text-sm font-medium">{item}</div>
                  </div>
                ))}
              </div>
            </div>
          </div>
        </CardContent>
      </Card>
      <p className="text-center text-xs text-muted-foreground">
        当前环境默认账号 admin / admin123
      </p>
    </div>
  );
}
