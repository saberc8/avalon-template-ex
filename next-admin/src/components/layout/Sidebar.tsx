"use client";

import { Box, Flex, Text, Link as ChakraLink, VStack } from "@chakra-ui/react";
import NextLink from "next/link";
import { usePathname } from "next/navigation";
import { menuItems, type AppMenuItem } from "@/config/menu";

function MenuItem({ item }: { item: AppMenuItem }) {
  const pathname = usePathname();

  if (item.children && item.children.length > 0) {
    return (
      <Box mb={4}>
        <Text fontSize="sm" fontWeight="bold" color="gray.500" px={4} mb={2}>
          {item.label}
        </Text>
        <VStack align="stretch" spacing={1}>
          {item.children.map((child) => {
            const isActive = child.path && pathname.startsWith(child.path);
            return (
              <ChakraLink
                key={child.path}
                as={NextLink}
                href={child.path ?? "#"}
                px={4}
                py={2}
                borderRadius="md"
                bg={isActive ? "blue.500" : "transparent"}
                color={isActive ? "white" : "gray.700"}
                _hover={{ bg: isActive ? "blue.600" : "gray.100" }}
                fontSize="sm"
              >
                {child.label}
              </ChakraLink>
            );
          })}
        </VStack>
      </Box>
    );
  }

  const isActive = item.path && pathname.startsWith(item.path);

  return (
    <ChakraLink
      as={NextLink}
      href={item.path ?? "#"}
      px={4}
      py={2}
      borderRadius="md"
      bg={isActive ? "blue.500" : "transparent"}
      color={isActive ? "white" : "gray.700"}
      _hover={{ bg: isActive ? "blue.600" : "gray.100" }}
      fontSize="sm"
      display="block"
    >
      {item.label}
    </ChakraLink>
  );
}

export function Sidebar() {
  return (
    <Box
      as="aside"
      width="230px"
      borderRightWidth="1px"
      borderColor="gray.200"
      bg="white"
      display={{ base: "none", md: "flex" }}
      flexDirection="column"
      overflow="hidden"
    >
      <Flex
        align="center"
        justify="center"
        height="56px"
        borderBottomWidth="1px"
        borderColor="gray.200"
        fontWeight="bold"
      >
        ContiNew Admin
      </Flex>
      <Box flex="1" overflowY="auto" py={4}>
        <VStack align="stretch" spacing={1}>
          {menuItems.map((item) => (
            <MenuItem key={item.label} item={item} />
          ))}
        </VStack>
      </Box>
    </Box>
  );
}
