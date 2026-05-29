"use client";

import { LogOut, UserRound } from "lucide-react";
import Link from "next/link";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger
} from "@/components/ui/dropdown-menu";
import { useCurrentUser } from "@/hooks/use-current-user";

export function UserMenu() {
  const { user, logout } = useCurrentUser();
  const displayName = user?.nickname || user?.username || "用户";
  const fallback = displayName.slice(0, 1).toUpperCase();

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button className="h-9 gap-2 px-2" variant="ghost">
          <Avatar className="size-7">
            {user?.avatar ? <AvatarImage src={user.avatar} alt={displayName} /> : null}
            <AvatarFallback className="text-xs">{fallback}</AvatarFallback>
          </Avatar>
          <span className="hidden max-w-28 truncate text-sm md:inline">{displayName}</span>
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="w-48">
        <DropdownMenuLabel>
          <div className="truncate">{displayName}</div>
          <div className="truncate text-xs font-normal text-muted-foreground">{user?.deptName || "-"}</div>
        </DropdownMenuLabel>
        <DropdownMenuSeparator />
        <DropdownMenuItem asChild>
          <Link href="/user/profile">
            <UserRound className="size-4" />
            个人中心
          </Link>
        </DropdownMenuItem>
        <DropdownMenuItem onClick={() => void logout()}>
          <LogOut className="size-4" />
          退出登录
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
