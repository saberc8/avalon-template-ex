<?php

declare(strict_types=1);

namespace Voc\Admin\Application\Auth;

use Voc\Admin\Domain\Rbac\Menu;

/**
 * 根据菜单列表构造前端使用的路由树结构。
 */
final class RouteBuilder
{
    /**
     * @param Menu[]   $menus
     * @param string[] $roles
     * @return array[]
     */
    public static function buildRouteTree(array $menus, array $roles): array
    {
        // 过滤掉按钮类型（type = 3）
        $filtered = [];
        foreach ($menus as $m) {
            if ($m->type === 3) {
                continue;
            }
            $filtered[] = $m;
        }
        if (!$filtered) {
            return [];
        }

        // 按 sort 升序，再按 id 升序
        usort($filtered, function (Menu $a, Menu $b): int {
            if ($a->sort === $b->sort) {
                return $a->id <=> $b->id;
            }
            return $a->sort <=> $b->sort;
        });

        // 构造 id -> 节点 映射
        /** @var array<int, array<string,mixed>> $nodeMap */
        $nodeMap = [];
        foreach ($filtered as $m) {
            $nodeMap[$m->id] = [
                'id'         => $m->id,
                'title'      => $m->title,
                'parentId'   => $m->parentId,
                'type'       => $m->type,
                'path'       => $m->path,
                'name'       => $m->name,
                'component'  => $m->component,
                'redirect'   => $m->redirect,
                'icon'       => $m->icon,
                'isExternal' => $m->isExternal,
                'isHidden'   => $m->isHidden,
                'isCache'    => $m->isCache,
                'permission' => $m->permission,
                'roles'      => $roles,
                'sort'       => $m->sort,
                'status'     => $m->status,
                'children'   => [],
                'activeMenu' => '',
                'alwaysShow' => false,
                'breadcrumb' => true,
                'showInTabs' => true,
                'affix'      => false,
            ];
        }

        // 建立父子关系
        foreach ($nodeMap as $id => &$node) {
            $parentId = $node['parentId'];
            if ($parentId === 0) {
                continue;
            }
            if (isset($nodeMap[$parentId])) {
                $nodeMap[$parentId]['children'][] = &$node;
            }
        }
        unset($node);

        // 收集根节点（或孤儿节点）
        $roots = [];
        foreach ($nodeMap as $id => $node) {
            $parentId = $node['parentId'];
            if ($parentId === 0 || !isset($nodeMap[$parentId])) {
                $roots[] = $node;
            }
        }

        // 递归对子节点排序
        $sortChildren = function (&$nodes) use (&$sortChildren) {
            foreach ($nodes as &$n) {
                if (!empty($n['children'])) {
                    usort($n['children'], function ($a, $b): int {
                        if ($a['sort'] === $b['sort']) {
                            return $a['id'] <=> $b['id'];
                        }
                        return $a['sort'] <=> $b['sort'];
                    });
                    $sortChildren($n['children']);
                }
            }
        };
        $sortChildren($roots);

        return $roots;
    }
}

