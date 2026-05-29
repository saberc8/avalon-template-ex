import { Badge } from "@/components/ui/badge";

export function StatusBadge({ status }: { status: number }) {
  return status === 1 ? (
    <Badge className="bg-secondary text-secondary-foreground">启用</Badge>
  ) : (
    <Badge variant="destructive">禁用</Badge>
  );
}
