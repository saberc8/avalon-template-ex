<?php

declare(strict_types=1);

namespace Voc\Admin\Domain\Rbac;

/**
 * 菜单实体，对应 sys_menu，供用户路由构造使用。
 */
final class Menu
{
    public int $id;
    public string $title;
    public int $parentId;
    public int $type;
    public string $path;
    public string $name;
    public string $component;
    public string $redirect;
    public string $icon;
    public bool $isExternal;
    public bool $isCache;
    public bool $isHidden;
    public string $permission;
    public int $sort;
    public int $status;

    public static function fromRow(array $row): self
    {
        $m = new self();
        $m->id = (int)$row['id'];
        $m->title = (string)$row['title'];
        $m->parentId = (int)$row['parent_id'];
        $m->type = (int)$row['type'];
        $m->path = (string)($row['path'] ?? '');
        $m->name = (string)($row['name'] ?? '');
        $m->component = (string)($row['component'] ?? '');
        $m->redirect = (string)($row['redirect'] ?? '');
        $m->icon = (string)($row['icon'] ?? '');
        $m->isExternal = (bool)($row['is_external'] ?? false);
        $m->isCache = (bool)($row['is_cache'] ?? false);
        $m->isHidden = (bool)($row['is_hidden'] ?? false);
        $m->permission = (string)($row['permission'] ?? '');
        $m->sort = (int)$row['sort'];
        $m->status = (int)$row['status'];
        return $m;
    }
}

