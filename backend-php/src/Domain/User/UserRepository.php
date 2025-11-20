<?php

declare(strict_types=1);

namespace Voc\Admin\Domain\User;

use PDO;

/**
 * 用户仓储，基于 sys_user 表。
 */
final class UserRepository
{
    private PDO $db;

    public function __construct(PDO $db)
    {
        $this->db = $db;
    }

    public function findByUsername(string $username): ?User
    {
        $sql = <<<SQL
SELECT
    id,
    username,
    nickname,
    password,
    gender,
    email,
    phone,
    avatar,
    description,
    status,
    is_system,
    pwd_reset_time,
    dept_id,
    create_time
FROM sys_user
WHERE username = :username
LIMIT 1
SQL;

        $stmt = $this->db->prepare($sql);
        $stmt->execute([':username' => $username]);
        $row = $stmt->fetch();
        if ($row === false) {
            return null;
        }
        return User::fromRow($row);
    }

    public function findById(int $id): ?User
    {
        $sql = <<<SQL
SELECT
    id,
    username,
    nickname,
    password,
    gender,
    email,
    phone,
    avatar,
    description,
    status,
    is_system,
    pwd_reset_time,
    dept_id,
    create_time
FROM sys_user
WHERE id = :id
LIMIT 1
SQL;

        $stmt = $this->db->prepare($sql);
        $stmt->execute([':id' => $id]);
        $row = $stmt->fetch();
        if ($row === false) {
            return null;
        }
        return User::fromRow($row);
    }
}

