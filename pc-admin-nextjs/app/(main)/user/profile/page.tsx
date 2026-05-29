"use client";

import { Camera, Link2, Save, Shield, Unlink } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { toast } from "sonner";
import {
  bindSocialAccount,
  listUserSocial,
  unbindSocialAccount,
  updateUserBaseInfo,
  updateUserEmail,
  updateUserPassword,
  updateUserPhone,
  uploadAvatar
} from "@/api/system/user-profile";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
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
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { useCurrentUser } from "@/hooks/use-current-user";
import type { SocialAccountResp } from "@/types/profile";

export default function ProfilePage() {
  const { user, reload } = useCurrentUser();
  const [nickname, setNickname] = useState(user?.nickname ?? "");
  const [gender, setGender] = useState(String(user?.gender ?? 0));
  const [oldPassword, setOldPassword] = useState("");
  const [newPassword, setNewPassword] = useState("");
  const [phone, setPhone] = useState(user?.phone ?? "");
  const [email, setEmail] = useState(user?.email ?? "");
  const [captcha, setCaptcha] = useState("local");
  const [socials, setSocials] = useState<SocialAccountResp[]>([]);
  const fileRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    setNickname(user?.nickname ?? "");
    setGender(String(user?.gender ?? 0));
    setPhone(user?.phone ?? "");
    setEmail(user?.email ?? "");
  }, [user]);

  useEffect(() => {
    void loadSocials();
  }, []);

  async function loadSocials() {
    try {
      setSocials(await listUserSocial());
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "三方账号加载失败");
    }
  }

  async function saveBasicInfo() {
    try {
      await updateUserBaseInfo({ nickname, gender: Number(gender) });
      await reload();
      toast.success("基础信息已保存");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "基础信息保存失败");
    }
  }

  async function savePassword() {
    try {
      await updateUserPassword({ oldPassword, newPassword });
      setOldPassword("");
      setNewPassword("");
      toast.success("密码已更新");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "密码更新失败");
    }
  }

  async function savePhone() {
    try {
      await updateUserPhone({ phone, captcha, oldPassword });
      await reload();
      toast.success("手机号已更新");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "手机号更新失败");
    }
  }

  async function saveEmail() {
    try {
      await updateUserEmail({ email, captcha, oldPassword });
      await reload();
      toast.success("邮箱已更新");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "邮箱更新失败");
    }
  }

  async function upload(file: File) {
    const data = new FormData();
    data.append("avatarFile", file);
    try {
      await uploadAvatar(data);
      await reload();
      toast.success("头像已更新");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "头像上传失败");
    }
  }

  async function toggleSocial(source: string, linked: boolean) {
    try {
      if (linked) await unbindSocialAccount(source);
      else await bindSocialAccount(source);
      await loadSocials();
      toast.success(linked ? "已解绑" : "已绑定");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "三方账号操作失败");
    }
  }

  const linkedSources = socials.map((item) => item.source);

  return (
    <div className="mx-auto grid w-full max-w-5xl gap-4 lg:grid-cols-[280px_1fr]">
      <section className="rounded-lg border bg-background p-4">
        <div className="flex flex-col items-center gap-3">
          <Avatar className="size-24">
            {user?.avatar ? <AvatarImage src={user.avatar} alt={user.nickname} /> : null}
            <AvatarFallback className="text-2xl">{(user?.nickname || user?.username || "U").slice(0, 1)}</AvatarFallback>
          </Avatar>
          <input
            ref={fileRef}
            className="hidden"
            type="file"
            accept="image/*"
            onChange={(event) => {
              const file = event.target.files?.[0];
              if (file) void upload(file);
              event.target.value = "";
            }}
          />
          <Button variant="outline" onClick={() => fileRef.current?.click()}>
            <Camera />
            更换头像
          </Button>
          <div className="w-full rounded-md border p-3 text-sm">
            <div className="font-medium">{user?.username}</div>
            <div className="text-muted-foreground">{user?.deptName || "-"}</div>
          </div>
        </div>
      </section>

      <section className="rounded-lg border bg-background p-4">
        <Tabs defaultValue="basic">
          <TabsList className="mb-4">
            <TabsTrigger value="basic">基础信息</TabsTrigger>
            <TabsTrigger value="password">密码</TabsTrigger>
            <TabsTrigger value="contact">联系方式</TabsTrigger>
            <TabsTrigger value="social">三方账号</TabsTrigger>
          </TabsList>
          <TabsContent value="basic" className="grid gap-4">
            <Field label="昵称">
              <Input value={nickname} onChange={(event) => setNickname(event.target.value)} />
            </Field>
            <Field label="性别">
              <Select value={gender} onValueChange={setGender}>
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
            <Button className="w-fit" onClick={() => void saveBasicInfo()}>
              <Save />
              保存
            </Button>
          </TabsContent>
          <TabsContent value="password" className="grid gap-4">
            <Field label="旧密码">
              <Input value={oldPassword} type="password" onChange={(event) => setOldPassword(event.target.value)} />
            </Field>
            <Field label="新密码">
              <Input value={newPassword} type="password" onChange={(event) => setNewPassword(event.target.value)} />
            </Field>
            <Button className="w-fit" onClick={() => void savePassword()}>
              <Shield />
              更新密码
            </Button>
          </TabsContent>
          <TabsContent value="contact" className="grid gap-4">
            <Field label="验证码">
              <Input value={captcha} onChange={(event) => setCaptcha(event.target.value)} />
            </Field>
            <Field label="手机号">
              <Input value={phone} onChange={(event) => setPhone(event.target.value)} />
            </Field>
            <Button className="w-fit" onClick={() => void savePhone()}>
              <Save />
              保存手机号
            </Button>
            <Field label="邮箱">
              <Input value={email} type="email" onChange={(event) => setEmail(event.target.value)} />
            </Field>
            <Button className="w-fit" onClick={() => void saveEmail()}>
              <Save />
              保存邮箱
            </Button>
          </TabsContent>
          <TabsContent value="social" className="grid gap-3">
            {["github", "gitee", "wechat"].map((source) => {
              const linked = linkedSources.includes(source);
              return (
                <div key={source} className="flex items-center justify-between rounded-md border p-3">
                  <span className="font-medium">{source}</span>
                  <Button variant="outline" onClick={() => void toggleSocial(source, linked)}>
                    {linked ? <Unlink /> : <Link2 />}
                    {linked ? "解绑" : "绑定"}
                  </Button>
                </div>
              );
            })}
          </TabsContent>
        </Tabs>
      </section>
    </div>
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
