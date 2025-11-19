import { Metadata } from "next";
import { Box, Heading, Text } from "@chakra-ui/react";

export const metadata: Metadata = {
  title: "个人中心 - Next Admin",
};

export default function UserProfilePage() {
  return (
    <Box>
      <Heading size="md" mb={4}>
        个人中心
      </Heading>
      <Text fontSize="sm" color="gray.600">
        这里是个人中心页面，占位实现。可以在此展示用户基本信息、修改密码等功能。
      </Text>
    </Box>
  );
}
