"use client";

import {
  Box,
  Flex,
  Text,
  Link as ChakraLink,
  VStack,
  Center,
  Spinner,
  useToast,
} from "@chakra-ui/react";
import NextLink from "next/link";
import { useEffect, useState } from "react";
import { usePathname } from "next/navigation";
import { get } from "@/utils/api";
import type { RouteItem } from "@/types/route";
import {
  buildMenuItemsFromRoutes,
  fallbackMenuItems,
  type AppMenuItem,
} from "@/config/menu";

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
            const isActive =
              !!child.path && pathname.startsWith(child.path || "");
            return (
              <ChakraLink
                key={child.path ?? child.label}
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

  const isActive = !!item.path && pathname.startsWith(item.path || "");

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
  const [menus, setMenus] = useState<AppMenuItem[]>(fallbackMenuItems);
  const [loading, setLoading] = useState(false);
  const toast = useToast();

  useEffect(() => {
    let cancelled = false;

    const loadMenus = async () => {
      setLoading(true);
      try {
        const res = await get<RouteItem[]>("/auth/user/route");
        if (cancelled) return;
        const dynamicMenus = buildMenuItemsFromRoutes(res.data);
        setMenus(dynamicMenus.length ? dynamicMenus : fallbackMenuItems);
      } catch (error: any) {
        if (cancelled) return;
        console.error("加载菜单失败", error);
        toast({
          description: error?.message || "加载菜单失败",
          status: "error",
        });
        setMenus(fallbackMenuItems);
      } finally {
        if (!cancelled) {
          setLoading(false);
        }
      }
    };

    loadMenus();

    return () => {
      cancelled = true;
    };
  }, [toast]);

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
        {loading ? (
          <Center py={4}>
            <Spinner size="sm" />
          </Center>
        ) : (
          <VStack align="stretch" spacing={1}>
            {menus.map((item) => (
              <MenuItem key={item.label} item={item} />
            ))}
          </VStack>
        )}
      </Box>
    </Box>
  );
}
