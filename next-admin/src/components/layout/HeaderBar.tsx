"use client";

import {
  Flex,
  IconButton,
  Text,
  Avatar,
  HStack,
  Button,
  useColorMode,
  useToast,
} from "@chakra-ui/react";
import { SunIcon, MoonIcon } from "@chakra-ui/icons";
import { useRouter } from "next/navigation";
import { post } from "@/utils/api";
import { clearToken } from "@/utils/auth";

export function HeaderBar() {
  const { colorMode, toggleColorMode } = useColorMode();
  const router = useRouter();
  const toast = useToast();

  const handleLogout = async () => {
    try {
      await post("/auth/logout", {});
    } catch {
      // ignore error, still clear token
    } finally {
      clearToken();
      toast({ description: "已退出登录", status: "success" });
      router.replace("/login");
    }
  };

  return (
    <Flex
      as="header"
      height="56px"
      align="center"
      justify="space-between"
      px={4}
      borderBottomWidth="1px"
      borderColor="gray.200"
      bg="white"
    >
      <Text fontSize="lg" fontWeight="semibold">
        工作台
      </Text>
      <HStack spacing={4}>
        <IconButton
          aria-label="Toggle color mode"
          icon={colorMode === "light" ? <MoonIcon /> : <SunIcon />}
          variant="ghost"
          size="sm"
          onClick={toggleColorMode}
        />
        <Button size="sm" variant="outline" onClick={handleLogout}>
          退出登录
        </Button>
        <Avatar size="sm" name="管理员" />
      </HStack>
    </Flex>
  );
}
