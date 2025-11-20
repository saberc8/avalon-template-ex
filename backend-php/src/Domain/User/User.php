<?php

declare(strict_types=1);

namespace Voc\Admin\Domain\User;

/**
 * 用户领域实体，与 Go 版 User / sys_user 表结构保持一致的关键字段。
 */
final class User
{
    public int $id;
    public string $username;
    public string $nickname;
    public string $password;
    public int $gender;
    public ?string $email;
    public ?string $phone;
    public ?string $avatar;
    public ?string $description;
    public int $status;
    public bool $isSystem;
    public ?\DateTimeImmutable $pwdResetTime;
    public int $deptId;
    public \DateTimeImmutable $createTime;

    public static function fromRow(array $row): self
    {
        $u = new self();
        $u->id = (int)$row['id'];
        $u->username = (string)$row['username'];
        $u->nickname = (string)$row['nickname'];
        $u->password = (string)$row['password'];
        $u->gender = (int)$row['gender'];
        $u->email = $row['email'] ?? null;
        $u->phone = $row['phone'] ?? null;
        $u->avatar = $row['avatar'] ?? null;
        $u->description = $row['description'] ?? null;
        $u->status = (int)$row['status'];
        $u->isSystem = (bool)$row['is_system'];
        $u->deptId = (int)$row['dept_id'];
        $u->createTime = new \DateTimeImmutable($row['create_time']);

        $u->pwdResetTime = isset($row['pwd_reset_time']) && $row['pwd_reset_time'] !== null
            ? new \DateTimeImmutable($row['pwd_reset_time'])
            : null;

        return $u;
    }

    /**
     * 用户是否启用（status = 1）。
     */
    public function isEnabled(): bool
    {
        return $this->status === 1;
    }
}

