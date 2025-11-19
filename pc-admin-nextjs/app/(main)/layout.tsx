"use client";

import { ReactNode, useEffect, useState } from "react";
import {
  Flex,
  Box,
  useColorModeValue,
  Spinner,
  Center,
} from "@chakra-ui/react";
import { useRouter, usePathname } from "next/navigation";
import { Sidebar } from "@/components/layout/Sidebar";
import { HeaderBar } from "@/components/layout/HeaderBar";
import { getToken } from "@/utils/auth";

export default function MainLayout({ children }: { children: ReactNode }) {
  const bg = useColorModeValue("gray.50", "gray.900");
  const router = useRouter();
  const pathname = usePathname();
  const [ready, setReady] = useState(false);

  useEffect(() => {
    const token = getToken();
    if (!token) {
      const redirect =
        pathname && pathname !== "/login"
          ? `?redirect=${encodeURIComponent(pathname)}`
          : "";
      router.replace(`/login${redirect}`);
    } else {
      setReady(true);
    }
  }, [router, pathname]);

  if (!ready) {
    return (
      <Flex height="100vh" bg={bg} align="center" justify="center">
        <Center>
          <Spinner />
        </Center>
      </Flex>
    );
  }

  return (
    <Flex height="100vh" bg={bg} align="stretch" overflow="hidden">
      <Sidebar />
      <Flex direction="column" flex="1" overflow="hidden">
        <HeaderBar />
        <Box as="main" flex="1" p={4} overflowY="auto">
          {children}
        </Box>
      </Flex>
    </Flex>
  );
}
