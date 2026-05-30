"use client";

import type { ComponentType, HTMLAttributes } from "react";
import type { ButtonProps } from "@/components/ui/button";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

interface TableActionsProps extends HTMLAttributes<HTMLDivElement> {}

interface TableActionButtonProps extends Omit<ButtonProps, "children" | "size"> {
  icon: ComponentType<{ className?: string }>;
  label: string;
  destructive?: boolean;
}

export function TableActions({ className, ...props }: TableActionsProps) {
  return <div className={cn("flex flex-wrap items-center gap-1.5", className)} {...props} />;
}

export function TableActionButton({
  icon: Icon,
  label,
  destructive = false,
  className,
  variant = "ghost",
  ...props
}: TableActionButtonProps) {
  return (
    <Button
      size="sm"
      variant={variant}
      title={label}
      className={cn(
        "h-8 px-2 text-xs",
        destructive && "text-destructive hover:bg-destructive/10 hover:text-destructive",
        className
      )}
      {...props}
    >
      <Icon />
      <span>{label}</span>
    </Button>
  );
}
