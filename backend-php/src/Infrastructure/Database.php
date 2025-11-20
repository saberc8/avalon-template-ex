<?php

declare(strict_types=1);

namespace Voc\Admin\Infrastructure;

use PDO;
use PDOException;

/**
 * 数据库工厂与封装，使用 PDO 连接 PostgreSQL。
 * 与 Go 版 LoadConfigFromEnv / NewPostgres 环境变量保持一致。
 */
final class Database
{
    public static function createFromEnv(): PDO
    {
        $host = getenv('DB_HOST') ?: '127.0.0.1';
        $port = getenv('DB_PORT') ?: '5432';
        $user = getenv('DB_USER') ?: 'postgres';
        $password = getenv('DB_PWD') ?: '123456';
        $dbname = getenv('DB_NAME') ?: 'nv_admin';

        $dsn = sprintf('pgsql:host=%s;port=%s;dbname=%s', $host, $port, $dbname);

        try {
            $pdo = new PDO($dsn, $user, $password, [
                PDO::ATTR_ERRMODE            => PDO::ERRMODE_EXCEPTION,
                PDO::ATTR_DEFAULT_FETCH_MODE => PDO::FETCH_ASSOC,
                PDO::ATTR_EMULATE_PREPARES   => false,
            ]);
        } catch (PDOException $e) {
            // 这里直接抛出异常，由上层统一处理（启动失败时可快速发现问题）
            throw $e;
        }

        return $pdo;
    }
}

