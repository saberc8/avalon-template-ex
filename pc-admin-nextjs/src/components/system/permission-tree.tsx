"use client";

import { Checkbox } from "@/components/ui/checkbox";
import { flattenTree } from "@/lib/tree";
import type { MenuResp } from "@/types/system";

interface PermissionTreeProps {
  menus: MenuResp[];
  selectedIds: number[];
  onChange: (ids: number[]) => void;
}

export function PermissionTree({ menus, selectedIds, onChange }: PermissionTreeProps) {
  const rows = flattenTree(menus);

  return (
    <div className="max-h-96 overflow-auto rounded-md border">
      {rows.map(({ node, depth }) => (
        <label
          key={node.id}
          className="flex h-9 items-center gap-2 border-b px-3 text-sm last:border-b-0"
          style={{ paddingLeft: 12 + depth * 24 }}
        >
          <Checkbox
            checked={selectedIds.includes(node.id)}
            onCheckedChange={(checked) => {
              onChange(
                checked
                  ? [...selectedIds, node.id]
                  : selectedIds.filter((id) => id !== node.id)
              );
            }}
          />
          <span className="truncate">{node.title}</span>
          {node.permission ? (
            <span className="ml-auto truncate text-xs text-muted-foreground">{node.permission}</span>
          ) : null}
        </label>
      ))}
    </div>
  );
}
