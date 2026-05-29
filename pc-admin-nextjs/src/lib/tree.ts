export interface FlatTreeItem<T> {
  node: T;
  depth: number;
  parent: T | null;
}

export interface TreeLike<T> {
  children: T[];
}

export function flattenTree<T extends TreeLike<T>>(nodes: T[], depth = 0, parent: T | null = null) {
  const result: FlatTreeItem<T>[] = [];
  for (const node of nodes) {
    result.push({ node, depth, parent });
    result.push(...flattenTree(node.children, depth + 1, node));
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
