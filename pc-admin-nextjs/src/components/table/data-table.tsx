"use client";

import {
  flexRender,
  getCoreRowModel,
  useReactTable,
  type ColumnDef
} from "@tanstack/react-table";
import { Loader2 } from "lucide-react";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow
} from "@/components/ui/table";
import { cn } from "@/lib/utils";

interface DataTableProps<TData> {
  columns: ColumnDef<TData>[];
  data: TData[];
  loading?: boolean;
  emptyText?: string;
  className?: string;
}

export function DataTable<TData>({
  columns,
  data,
  loading = false,
  emptyText = "暂无数据",
  className
}: DataTableProps<TData>) {
  const table = useReactTable({
    data,
    columns,
    getCoreRowModel: getCoreRowModel()
  });
  const colSpan = Math.max(columns.length, 1);

  return (
    <div className={cn("overflow-x-auto rounded-lg border bg-background shadow-sm", className)}>
      <Table className="min-w-full">
        <TableHeader>
          {table.getHeaderGroups().map((headerGroup) => (
            <TableRow key={headerGroup.id} className="bg-muted/50 hover:bg-muted/50">
              {headerGroup.headers.map((header) => (
                <TableHead key={header.id} className="h-10 whitespace-nowrap text-xs font-medium">
                  {header.isPlaceholder
                    ? null
                    : flexRender(header.column.columnDef.header, header.getContext())}
                </TableHead>
              ))}
            </TableRow>
          ))}
        </TableHeader>
        <TableBody>
          {loading ? (
            <TableRow>
              <TableCell className="h-32 text-center text-muted-foreground" colSpan={colSpan}>
                <div className="inline-flex items-center gap-2 text-sm">
                  <Loader2 className="size-4 animate-spin" />
                  加载中
                </div>
              </TableCell>
            </TableRow>
          ) : table.getRowModel().rows.length > 0 ? (
            table.getRowModel().rows.map((row) => (
              <TableRow key={row.id} className="transition-colors hover:bg-muted/35">
                {row.getVisibleCells().map((cell) => (
                  <TableCell key={cell.id} className="h-[var(--table-row-height)] whitespace-nowrap py-2 text-sm">
                    {flexRender(cell.column.columnDef.cell, cell.getContext())}
                  </TableCell>
                ))}
              </TableRow>
            ))
          ) : (
            <TableRow>
              <TableCell className="h-32 text-center text-muted-foreground" colSpan={colSpan}>
                <div className="text-sm">{emptyText}</div>
              </TableCell>
            </TableRow>
          )}
        </TableBody>
      </Table>
    </div>
  );
}
