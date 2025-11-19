"use client";

import { ChakraProvider, extendTheme, ThemeConfig } from "@chakra-ui/react";
import { ReactNode } from "react";

const config: ThemeConfig = {
  initialColorMode: "light",
  useSystemColorMode: false,
};

const theme = extendTheme({
  config,
});

interface Props {
  children: ReactNode;
}

export function ChakraProviders({ children }: Props) {
  return <ChakraProvider theme={theme}>{children}</ChakraProvider>;
}
