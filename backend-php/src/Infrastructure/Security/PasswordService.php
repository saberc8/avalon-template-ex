<?php

declare(strict_types=1);

namespace Voc\Admin\Infrastructure\Security;

/**
 * 密码工具：兼容后端存储的 {bcrypt}$2a$... 形式。
 */
final class PasswordService
{
    /**
     * 校验明文密码与存储密码是否一致。
     */
    public function verify(string $raw, string $encoded): bool
    {
        $encoded = trim($encoded);
        if ($encoded === '') {
            return false;
        }

        // 去掉与 Spring Security 保持一致的 {bcrypt} 前缀
        $prefix = '{bcrypt}';
        if (str_starts_with($encoded, $prefix)) {
            $encoded = substr($encoded, strlen($prefix));
        }

        return password_verify($raw, $encoded);
    }

    /**
     * 生成 {bcrypt} 前缀的密码哈希，供新增用户 / 重置密码使用。
     */
    public function hash(string $raw): string
    {
        $raw = trim($raw);
        if ($raw === '') {
            throw new \InvalidArgumentException('密码不能为空');
        }

        $hash = password_hash($raw, PASSWORD_BCRYPT);
        if ($hash === false) {
            throw new \RuntimeException('密码加密失败');
        }

        return '{bcrypt}' . $hash;
    }
}

