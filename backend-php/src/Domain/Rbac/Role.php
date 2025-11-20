<?php

declare(strict_types=1);

namespace Voc\Admin\Domain\Rbac;

/**
 * 角色实体，对应 sys_role。
 */
final class Role
{
    public int $id;
    public string $name;
    public string $code;

    public static function fromRow(array $row): self
    {
        $r = new self();
        $r->id = (int)$row['id'];
        $r->name = (string)$row['name'];
        $r->code = (string)$row['code'];
        return $r;
    }
}

