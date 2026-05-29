"use client";

import { zodResolver } from "@hookform/resolvers/zod";
import { LockKeyhole, LogIn, ShieldCheck, User } from "lucide-react";
import { useRouter } from "next/navigation";
import { useEffect, useState } from "react";
import { useForm } from "react-hook-form";
import { toast } from "sonner";
import { z } from "zod";
import { accountLogin, getUserInfo, getUserRoutes } from "@/api/auth";
import { Button } from "@/components/ui/button";
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

const loginSchema = z.object({
  username: z.string().min(1, "请输入用户名"),
  password: z.string().min(1, "请输入密码"),
  captcha: z.string().optional()
});

type LoginValues = z.infer<typeof loginSchema>;

export function LoginForm() {
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
    <Form {...form}>
      <form className="space-y-4" onSubmit={form.handleSubmit(onSubmit)}>
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
      </form>
    </Form>
  );
}
