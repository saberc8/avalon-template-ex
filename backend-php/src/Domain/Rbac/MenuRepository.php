<?php

declare(strict_types=1);

namespace Voc\Admin\Domain\Rbac;

use PDO;

/**
 * 菜单仓储，用于按用户查询权限和路由菜单。
 */
final class MenuRepository
{
    private PDO $db;

    public function __construct(PDO $db)
    {
        $this->db = $db;
    }

    /**
     * 根据用户 ID 查询其所有菜单（去重）。
     *
     * @return Menu[]
     */
    public function listByUserId(int $userId): array
    {
        $sql = <<<SQL
SELECT DISTINCT m.*
FROM sys_menu m
JOIN sys_role_menu rm ON rm.menu_id = m.id
JOIN sys_user_role ur ON ur.role_id = rm.role_id
WHERE ur.user_id = :uid
SQL;

        $stmt = $this->db->prepare($sql);
        $stmt->execute([':uid' => $userId]);
        $rows = $stmt->fetchAll();
        $menus = [];
        foreach ($rows as $row) {
            $menus[] = Menu::fromRow($row);
        }
        return $menus;
    }

    /**
     * 根据用户 ID 查询权限码列表。
     *
     * @return string[]
     */
    public function listPermissionsByUserId(int $userId): array
    {
        $sql = <<<SQL
SELECT DISTINCT m.permission
FROM sys_menu m
JOIN sys_role_menu rm ON rm.menu_id = m.id
JOIN sys_user_role ur ON ur.role_id = rm.role_id
WHERE ur.user_id = :uid
  AND m.permission IS NOT NULL
  AND m.permission <> ''
SQL;

        $stmt = $this->db->prepare($sql);
        $stmt->execute([':uid' => $userId]);
        $rows = $stmt->fetchAll();
        $perms = [];
        foreach ($rows as $row) {
            $perms[] = (string)$row['permission'];
        }
        return $perms;
    }
}

