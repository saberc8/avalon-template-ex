"use client";

import { Box, Card, CardBody, CardHeader, Heading, SimpleGrid, Text } from "@chakra-ui/react";
import { useRouter } from "next/navigation";

const links = [
  { text: "用户管理", path: "/system/user" },
  { text: "角色管理", path: "/system/role" },
  { text: "菜单管理", path: "/system/menu" },
  { text: "文件管理", path: "/system/file" },
  { text: "系统日志", path: "/monitor/log" },
];

export function WorkplaceQuickOperation() {
  const router = useRouter();

  return (
    <Card variant="outline">
      <CardHeader pb={0}>
        <Heading size="sm">快捷操作</Heading>
      </CardHeader>
      <CardBody pt={4}>
        <SimpleGrid columns={3} spacing={3}>
          {links.map((link) => (
            <Box
              key={link.text}
              textAlign="center"
              cursor="pointer"
              onClick={() => router.push(link.path)}
              _hover={{
                ".icon": { bg: "blue.50", color: "blue.500" },
                ".text": { color: "blue.500" },
              }}
            >
              <Box
                className="icon"
                display="inline-flex"
                alignItems="center"
                justifyContent="center"
                w={8}
                h={8}
                mb={1}
                borderRadius="md"
                bg="gray.100"
                color="gray.600"
                fontSize="sm"
              >
                {/* 可以在这里接入图标库 */}
                {link.text.charAt(0)}
              </Box>
              <Text className="text" fontSize="xs" color="gray.600">
                {link.text}
              </Text>
            </Box>
          ))}
        </SimpleGrid>
      </CardBody>
    </Card>
  );
}
