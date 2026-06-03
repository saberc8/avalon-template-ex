export interface FlatTreeItem<T> {
  node: T;
  depth: number;
  parent: T | null;
}

export interface VisibleFlatTreeItem<T> extends FlatTreeItem<T> {
  hasChildren: boolean;
  expanded: boolean;
}

export interface TreeLike<T> {
  children: T[];
}

type TreeNodeId = string | number;

export function flattenTree<T extends TreeLike<T>>(nodes: T[], depth = 0, parent: T | null = null) {
  const result: FlatTreeItem<T>[] = [];
  for (const node of nodes) {
    result.push({ node, depth, parent });
    result.push(...flattenTree(node.children, depth + 1, node));
  }
  return result;
}

export function flattenVisibleTree<T extends TreeLike<T> & { id: TreeNodeId }>(
  nodes: T[],
  collapsedIds: ReadonlySet<TreeNodeId>,
  depth = 0,
  parent: T | null = null
) {
  const result: VisibleFlatTreeItem<T>[] = [];
  for (const node of nodes) {
    const hasChildren = node.children.length > 0;
    const expanded = hasChildren && !collapsedIds.has(node.id);
    result.push({ node, depth, parent, hasChildren, expanded });
    if (expanded) {
      result.push(...flattenVisibleTree(node.children, collapsedIds, depth + 1, node));
    }
  }
  return result;
}

export function mapTree<T extends TreeLike<T>, R extends TreeLike<R>>(
  nodes: T[],
  mapper: (node: T) => Omit<R, "children"> & { children?: unknown }
): R[] {
  return nodes.map((node) => {
    const mapped = mapper(node);
    return {
      ...mapped,
      children: mapTree(node.children, mapper)
    } as R;
  });
}
