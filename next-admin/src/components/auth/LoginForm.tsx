"use client";

import { useCallback, useEffect, useRef, useState } from "react";
import {
  Box,
  Button,
  Checkbox,
  Flex,
  Heading,
  HStack,
  Image,
  Input,
  Stack,
  Text,
  useToast,
} from "@chakra-ui/react";
import { useRouter, useSearchParams } from "next/navigation";
import { get, post } from "@/utils/api";
import { encryptByRsa } from "@/utils/encrypt";
import { getToken, setToken } from "@/utils/auth";
import { goodTimeText } from "@/utils/time";

type ImageCaptchaResp = {
  uuid: string;
  img: string;
  expireTime: number;
  isEnabled: boolean;
};

type LoginResp = {
  token: string;
};

type LoginConfig = {
  rememberMe: boolean;
  username: string;
};

const LOGIN_CONFIG_KEY = "login-config";

export function LoginForm() {
  const toast = useToast();
  const router = useRouter();
  const searchParams = useSearchParams();

  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [captcha, setCaptcha] = useState("");
  const [uuid, setUuid] = useState("");
  const [rememberMe, setRememberMe] = useState(true);
  const [loading, setLoading] = useState(false);

  const [isCaptchaEnabled, setIsCaptchaEnabled] = useState(true);
  const [captchaImgBase64, setCaptchaImgBase64] = useState<string | null>(null);
  const [expired, setExpired] = useState(false);
  const timerRef = useRef<number | null>(null);

  // 已登录则直接跳转到重定向地址或首页
  useEffect(() => {
    const token = getToken();
    if (token) {
      const redirect = searchParams.get("redirect");
      router.replace(redirect || "/dashboard/workplace");
    }
  }, [router, searchParams]);

  // 初始化本地登录配置（记住我 & 默认演示账号）
  useEffect(() => {
    if (typeof window === "undefined") return;
    try {
      const stored = window.localStorage.getItem(LOGIN_CONFIG_KEY);
      if (stored) {
        const parsed: LoginConfig = JSON.parse(stored);
        setRememberMe(parsed.rememberMe ?? true);
        setUsername(parsed.username ?? "");
        // 为了和 Vue 版本体验接近，这里仍给出默认演示密码
        setPassword("admin123");
        return;
      }
    } catch {
      // ignore parse errors
    }
    // 默认演示账号
    setRememberMe(true);
    setUsername("admin");
    setPassword("admin123");
  }, []);

  const clearTimer = () => {
    if (timerRef.current) {
      window.clearTimeout(timerRef.current);
      timerRef.current = null;
    }
  };

  const startTimer = useCallback((expireTime: number, serverTime: number) => {
    clearTimer();
    const remaining = expireTime - serverTime;
    if (remaining <= 0) {
      setExpired(true);
      return;
    }
    timerRef.current = window.setTimeout(() => {
      setExpired(true);
    }, remaining);
  }, []);

  const fetchCaptcha = useCallback(async () => {
    try {
      const res = await get<ImageCaptchaResp>("/captcha/image");
      const { uuid, img, expireTime, isEnabled } = res.data;
      setIsCaptchaEnabled(isEnabled);
      setCaptchaImgBase64(img);
      setUuid(uuid);
      setExpired(false);
      const serverTime = Number(res.timestamp);
      if (!Number.isNaN(serverTime)) {
        startTimer(expireTime, serverTime);
      }
    } catch (error: any) {
      toast({
        description: error?.message || "获取验证码失败",
        status: "error",
      });
    }
  }, [startTimer, toast]);

  useEffect(() => {
    fetchCaptcha();
    return () => {
      clearTimer();
    };
  }, [fetchCaptcha]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!username) {
      toast({ description: "请输入用户名", status: "warning" });
      return;
    }
    if (!password) {
      toast({ description: "请输入密码", status: "warning" });
      return;
    }
    if (isCaptchaEnabled) {
      if (!captcha) {
        toast({ description: "请输入验证码", status: "warning" });
        return;
      }
      if (expired) {
        toast({ description: "验证码已过期，请刷新", status: "warning" });
        return;
      }
    }

    setLoading(true);
    try {
      const encrypted = encryptByRsa(password) || "";
      const clientId =
        process.env.NEXT_PUBLIC_CLIENT_ID ??
        "ef51c9a3e9046c4f2ea45142c8a8344a";

      const body = {
        username,
        password: encrypted,
        captcha,
        uuid,
        clientId,
        authType: "ACCOUNT" as const,
      };

      const res = await post<LoginResp, typeof body>("/auth/login", body);
      setToken(res.data.token);

      if (typeof window !== "undefined") {
        const config: LoginConfig = {
          rememberMe,
          username: rememberMe ? username : "",
        };
        window.localStorage.setItem(LOGIN_CONFIG_KEY, JSON.stringify(config));
      }

      const redirect = searchParams.get("redirect");
      const greeting = goodTimeText();
      toast({
        description: `${greeting}，欢迎使用`,
        status: "success",
      });
      router.push(redirect || "/dashboard/workplace");
    } catch (error: any) {
      toast({
        description: error?.message || "登录失败，请重试",
        status: "error",
      });
      setCaptcha("");
      fetchCaptcha();
    } finally {
      setLoading(false);
    }
  };

  return (
    <Flex minH="100vh" align="center" justify="center" bg="gray.50">
      <Box bg="white" p={8} rounded="lg" shadow="md" width="full" maxW="md">
        <Heading mb={6} size="md" textAlign="center">
          管理后台登录
        </Heading>
        <Box as="form" onSubmit={handleSubmit}>
          <Stack spacing={4}>
            <Box>
              <Text mb={1} fontSize="sm">
                用户名
              </Text>
              <Input
                placeholder="请输入用户名"
                value={username}
                onChange={(e) => setUsername(e.target.value)}
              />
            </Box>
            <Box>
              <Text mb={1} fontSize="sm">
                密码
              </Text>
              <Input
                type="password"
                placeholder="请输入密码"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
              />
            </Box>

            {isCaptchaEnabled && (
              <HStack align="flex-end" spacing={3}>
                <Box flex="1">
                  <Text mb={1} fontSize="sm">
                    验证码
                  </Text>
                  <Input
                    placeholder="请输入验证码"
                    maxLength={4}
                    value={captcha}
                    onChange={(e) => setCaptcha(e.target.value)}
                  />
                </Box>
                <Box
                  position="relative"
                  cursor="pointer"
                  onClick={fetchCaptcha}
                >
                  {captchaImgBase64 && (
                    <Image
                      src={captchaImgBase64}
                      alt="验证码"
                      width="120px"
                      height="40px"
                      borderRadius="md"
                      objectFit="cover"
                    />
                  )}
                  {expired && (
                    <Flex
                      position="absolute"
                      top={0}
                      left={0}
                      right={0}
                      bottom={0}
                      bg="blackAlpha.600"
                      align="center"
                      justify="center"
                    >
                      <Text fontSize="xs" color="white">
                        已过期，请刷新
                      </Text>
                    </Flex>
                  )}
                </Box>
              </HStack>
            )}

            <Flex justify="space-between" align="center">
              <Checkbox
                isChecked={rememberMe}
                onChange={(e) => setRememberMe(e.target.checked)}
              >
                记住我
              </Checkbox>
              <Text fontSize="sm" color="blue.500" cursor="pointer">
                忘记密码
              </Text>
            </Flex>

            <Button
              colorScheme="blue"
              w="full"
              type="submit"
              isLoading={loading}
            >
              立即登录
            </Button>
          </Stack>
        </Box>
      </Box>
    </Flex>
  );
}

export default LoginForm;
