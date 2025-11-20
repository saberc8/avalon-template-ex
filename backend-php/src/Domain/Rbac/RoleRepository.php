<?php

declare(strict_types=1);

namespace Voc\Admin\Domain\Rbac;

use PDO;

/**
 * 角色仓储，用于按用户查询角色列表。
 */
final class RoleRepository
{
    private PDO $db;

    public function __construct(PDO $db)
    {
        $this->db = $db;
    }

    /**
     * 根据用户 ID 查询其角色列表。
     *
     * @return Role[]
     */
    public function listByUserId(int $userId): array
    {
        $sql = <<<SQL
SELECT r.id, r.name, r.code
FROM sys_role r
JOIN sys_user_role ur ON ur.role_id = r.id
WHERE ur.user_id = :uid
SQL;

        $stmt = $this->db->prepare($sql);
        $stmt->execute([':uid' => $userId]);
        $rows = $stmt->fetchAll();
        $roles = [];
        foreach ($rows as $row) {
            $roles[] = Role::fromRow($row);
        }
        return $roles;
    }
}

