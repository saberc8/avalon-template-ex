<?php

declare(strict_types=1);

namespace Voc\Admin\Interfaces\Http;

use PDO;
use PDOException;
use Psr\Http\Message\ResponseInterface;
use Psr\Http\Message\ServerRequestInterface;
use Slim\App;
use Voc\Admin\Infrastructure\Response\JsonResponder;
use Voc\Admin\Infrastructure\Security\TokenService;

/**
 * 系统参数管理路由：/system/option 系列接口。
 *
 * 目标：与 Java / Node 版本 OptionController 行为保持一致，
 * 支持列表查询、批量更新和恢复默认值。
 */
final class OptionRoutes
{
    private PDO $db;
    private TokenService $tokens;
    private JsonResponder $responder;

    public function __construct(PDO $db, TokenService $tokens, JsonResponder $responder)
    {
        $this->db = $db;
        $this->tokens = $tokens;
        $this->responder = $responder;
    }

    /**
     * 注册路由。
     */
    public function register(App $app): void
    {
        // 查询参数列表：GET /system/option
        $app->get('/system/option', fn (ServerRequestInterface $r) => $this->listOption($r));

        // 批量保存参数：PUT /system/option
        $app->put('/system/option', fn (ServerRequestInterface $r) => $this->updateOption($r));

        // 恢复默认值：PATCH /system/option/value
        $app->patch('/system/option/value', fn (ServerRequestInterface $r) => $this->resetOptionValue($r));
    }

    /**
     * 查询系统配置列表。
     *
     * 支持：
     * - code 多值查询（重复参数或逗号分隔）
     * - category 按类别筛选
     */
    private function listOption(ServerRequestInterface $request): ResponseInterface
    {
        $params = $request->getQueryParams();
        $codes = $this->normalizeMultiValue($params['code'] ?? null);
        $category = trim((string)($params['category'] ?? ''));

        $where = 'WHERE 1=1';
        $args = [];

        if ($codes) {
            $placeholders = [];
            foreach ($codes as $idx => $code) {
                $key = ':c' . $idx;
                $placeholders[] = $key;
                $args[$key] = $code;
            }
            $where .= ' AND code IN (' . implode(',', $placeholders) . ')';
        }
        if ($category !== '') {
            $where .= ' AND category = :category';
            $args[':category'] = $category;
        }

        $sql = "
SELECT id,
       name,
       code,
       COALESCE(value, default_value, '') AS value,
       COALESCE(description, '')          AS description
FROM sys_option
{$where}
ORDER BY id ASC";

        try {
            $stmt = $this->db->prepare($sql);
            $stmt->execute($args);
            $rows = $stmt->fetchAll();
        } catch (PDOException $e) {
            return $this->responder->fail('500', '查询系统配置失败');
        }

        $list = [];
        foreach ($rows as $row) {
            $list[] = [
                'id'          => (int)$row['id'],
                'name'        => (string)$row['name'],
                'code'        => (string)$row['code'],
                'value'       => (string)$row['value'],
                'description' => (string)$row['description'],
            ];
        }

        return $this->responder->ok($list);
    }

    /**
     * 批量保存系统配置值。
     *
     * body: [{ id, code, value }]
     */
    private function updateOption(ServerRequestInterface $request): ResponseInterface
    {
        $authz = $request->getHeaderLine('Authorization');
        try {
            $claims = $this->tokens->parse($authz);
        } catch (\Throwable $e) {
            return $this->responder->fail('401', '未授权，请重新登录');
        }
        $userId = (int)$claims['userId'];
        if ($userId <= 0) {
            return $this->responder->fail('401', '未授权，请重新登录');
        }

        $data = json_decode((string)$request->getBody(), true);
        if (!is_array($data) || $data === []) {
            return $this->responder->fail('400', '请求参数不正确');
        }

        $now = (new \DateTimeImmutable())->format('Y-m-d H:i:s');

        try {
            $this->db->beginTransaction();
            $sql = "
UPDATE sys_option
   SET value       = :value,
       update_user = :update_user,
       update_time = :update_time
 WHERE id          = :id
   AND code        = :code";
            $stmt = $this->db->prepare($sql);

            foreach ($data as $item) {
                if (!is_array($item)) {
                    throw new \RuntimeException('invalid payload');
                }
                $id = (int)($item['id'] ?? 0);
                $code = trim((string)($item['code'] ?? ''));
                if ($id <= 0 || $code === '') {
                    throw new \RuntimeException('invalid payload');
                }
                $value = $this->toOptionValueString($item['value'] ?? null);

                $stmt->execute([
                    ':value'       => $value,
                    ':update_user' => $userId,
                    ':update_time' => $now,
                    ':id'          => $id,
                    ':code'        => $code,
                ]);
            }

            $this->db->commit();
        } catch (\RuntimeException $e) {
            $this->db->rollBack();
            return $this->responder->fail('400', '请求参数不正确');
        } catch (PDOException $e) {
            $this->db->rollBack();
            return $this->responder->fail('500', '保存系统配置失败');
        }

        return $this->responder->ok(true);
    }

    /**
     * 恢复默认值：PATCH /system/option/value。
     *
     * body: { category?: string, code?: string[] }
     * 至少指定其一。
     */
    private function resetOptionValue(ServerRequestInterface $request): ResponseInterface
    {
        $authz = $request->getHeaderLine('Authorization');
        try {
            $claims = $this->tokens->parse($authz);
        } catch (\Throwable $e) {
            return $this->responder->fail('401', '未授权，请重新登录');
        }
        $userId = (int)$claims['userId'];
        if ($userId <= 0) {
            return $this->responder->fail('401', '未授权，请重新登录');
        }

        $body = json_decode((string)$request->getBody(), true);
        if (!is_array($body)) {
            return $this->responder->fail('400', '请求参数不正确');
        }

        $category = trim((string)($body['category'] ?? ''));
        $codesRaw = $body['code'] ?? [];
        $codes = [];
        if (is_array($codesRaw)) {
            foreach ($codesRaw as $c) {
                $c = trim((string)$c);
                if ($c !== '') {
                    $codes[] = $c;
                }
            }
        }
        if ($category === '' && !$codes) {
            return $this->responder->fail('400', '键列表或类别不能为空');
        }

        $sql = 'UPDATE sys_option SET value = NULL';
        $args = [];

        if ($category !== '') {
            $sql .= ' WHERE category = :category';
            $args[':category'] = $category;
        } elseif ($codes) {
            $placeholders = [];
            foreach ($codes as $idx => $code) {
                $key = ':c' . $idx;
                $placeholders[] = $key;
                $args[$key] = $code;
            }
            $sql .= ' WHERE code IN (' . implode(',', $placeholders) . ')';
        }

        try {
            $this->db->beginTransaction();

            $stmt = $this->db->prepare($sql);
            $stmt->execute($args);

            // 可选：记录最近操作人
            $now = (new \DateTimeImmutable())->format('Y-m-d H:i:s');
            $auditSql = "
UPDATE sys_option
   SET update_user = :update_user,
       update_time = :update_time
 WHERE (category = :categoryCond OR :categoryCond = '')
    OR (code IN (" . ($codes ? implode(',', array_map(
                fn ($idx) => ':cAudit' . $idx,
                array_keys($codes)
            )) : "''") . '))';

            $auditArgs = [
                ':update_user'  => $userId,
                ':update_time'  => $now,
                ':categoryCond' => $category,
            ];
            if ($codes) {
                foreach ($codes as $idx => $code) {
                    $auditArgs[':cAudit' . $idx] = $code;
                }
            }
            $stmtAudit = $this->db->prepare($auditSql);
            $stmtAudit->execute($auditArgs);

            $this->db->commit();
        } catch (PDOException $e) {
            $this->db->rollBack();
            return $this->responder->fail('500', '恢复默认配置失败');
        }

        return $this->responder->ok(true);
    }

    /**
     * 解析 code 查询参数，兼容逗号分隔与重复参数形式。
     *
     * @param mixed $value
     * @return array<int,string>
     */
    private function normalizeMultiValue($value): array
    {
        if ($value === null) {
            return [];
        }

        $list = [];
        if (is_array($value)) {
            $list = $value;
        } else {
            $list = [$value];
        }

        $result = [];
        foreach ($list as $raw) {
            $raw = (string)$raw;
            foreach (explode(',', $raw) as $part) {
                $part = trim($part);
                if ($part !== '') {
                    $result[] = $part;
                }
            }
        }
        return $result;
    }

    /**
     * 将任意值转换为字符串，保持与 Java / Node 写入 sys_option 的逻辑一致。
     */
    private function toOptionValueString($value): string
    {
        if ($value === null) {
            return '';
        }
        if (is_string($value)) {
            return $value;
        }
        if (is_int($value) || is_float($value)) {
            if (!is_finite((float)$value)) {
                return '';
            }
            return (string)intval($value);
        }
        if (is_bool($value)) {
            return $value ? 'true' : 'false';
        }
        try {
            $encoded = json_encode($value, JSON_UNESCAPED_UNICODE);
            return is_string($encoded) ? $encoded : '';
        } catch (\Throwable $e) {
            return '';
        }
    }
}

