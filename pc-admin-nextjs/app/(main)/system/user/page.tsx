import { Box, Heading, Text } from "@chakra-ui/react";

export default function Page() {
  return (
    <Box>
      <Heading size="md" mb={4}>
        user 页面
      </Heading>
      <Text fontSize="sm" color="gray.600">
        这是从 Vue3 管理后台迁移过来的占位页面，后续可以补充具体功能。
      </Text>
    </Box>
  );
}
