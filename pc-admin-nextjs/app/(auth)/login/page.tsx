import { Metadata } from "next";
import { Box } from "@chakra-ui/react";
import { LoginForm } from "@/components/auth/LoginForm";

export const metadata: Metadata = {
  title: "登录 - Next Admin",
};

export default function LoginPage() {
  return (
    <Box>
      <LoginForm />
    </Box>
  );
}
