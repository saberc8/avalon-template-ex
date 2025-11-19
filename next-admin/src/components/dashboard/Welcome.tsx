"use client";

import { Avatar, Box, Flex, Text } from "@chakra-ui/react";
import { goodTimeText } from "@/utils/time";

export function WorkplaceWelcome() {
  const greeting = goodTimeText();
  const nickname = "管理员"; // TODO: 可从用户上下文或接口中获取

  return (
    <Box>
      <Flex align="center">
        <Avatar name={nickname} size="lg" mr={4} />
        <Box>
          <Text fontSize="xl" fontWeight="semibold" mb={1}>
            {greeting}！{nickname}
          </Text>
          <Text fontSize="sm" color="gray.500">
            北海虽赊，扶摇可接；东隅已逝，桑榆非晚。
          </Text>
        </Box>
      </Flex>
    </Box>
  );
}
