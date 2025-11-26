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
 * 字典管理路由：/system/dict 与 /system/dict/item。
 *
 * 目标：对齐 Java / Node 版本 SystemDictController 行为，
 * 支持字典及字典项的增删改查。
 */
final class DictRoutes
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
        // 字典列表（非分页）：GET /system/dict/list
        $app->get('/system/dict/list', fn (ServerRequestInterface $r) => $this->listDict($r));

        // 字典项分页查询：GET /system/dict/item
        $app->get('/system/dict/item', fn (ServerRequestInterface $r) => $this->listDictItem($r));

        // 字典项详情：GET /system/dict/item/{id}
        $app->get('/system/dict/item/{id}', fn (ServerRequestInterface $r, array $a) => $this->getDictItem($r, $a));

        // 新增字典项：POST /system/dict/item
        $app->post('/system/dict/item', fn (ServerRequestInterface $r) => $this->createDictItem($r));

        // 修改字典项：PUT /system/dict/item/{id}
        $app->put('/system/dict/item/{id}', fn (ServerRequestInterface $r, array $a) => $this->updateDictItem($r, $a));

        // 删除字典项：DELETE /system/dict/item
        $app->delete('/system/dict/item', fn (ServerRequestInterface $r) => $this->deleteDictItem($r));

        // 单个字典详情：GET /system/dict/{id}
        $app->get('/system/dict/{id}', fn (ServerRequestInterface $r, array $a) => $this->getDict($r, $a));

        // 新增字典：POST /system/dict
        $app->post('/system/dict', fn (ServerRequestInterface $r) => $this->createDict($r));

        // 修改字典：PUT /system/dict/{id}
        $app->put('/system/dict/{id}', fn (ServerRequestInterface $r, array $a) => $this->updateDict($r, $a));

        // 删除字典：DELETE /system/dict
        $app->delete('/system/dict', fn (ServerRequestInterface $r) => $this->deleteDict($r));

        // 清理字典缓存：DELETE /system/dict/cache/{code}（当前无实际缓存逻辑）
        $app->delete('/system/dict/cache/{code}', fn (ServerRequestInterface $r, array $a) => $this->clearDictCache($r, $a));
    }

    /**
     * 解析当前登录用户 ID。
     */
    private function currentUserId(ServerRequestInterface $request): int
    {
        $authz = $request->getHeaderLine('Authorization');
        try {
            $claims = $this->tokens->parse($authz);
        } catch (\Throwable $e) {
            throw new \RuntimeException('未授权，请重新登录');
        }
        return (int)$claims['userId'];
    }

    /**
     * 查询字典列表：GET /system/dict/list。
     */
    private function listDict(ServerRequestInterface $request): ResponseInterface
    {
        $params = $request->getQueryParams();
        $desc = trim((string)($params['description'] ?? ''));

        $sql = "
SELECT d.id,
       d.name,
       d.code,
       COALESCE(d.description, '')         AS description,
       COALESCE(d.is_system, FALSE)        AS is_system,
       d.create_time,
       COALESCE(cu.nickname, '')           AS create_user_string,
       d.update_time,
       COALESCE(uu.nickname, '')           AS update_user_string
FROM sys_dict AS d
LEFT JOIN sys_user AS cu ON cu.id = d.create_user
LEFT JOIN sys_user AS uu ON uu.id = d.update_user
ORDER BY d.create_time DESC, d.id DESC";

        try {
            $stmt = $this->db->query($sql);
            $rows = $stmt->fetchAll();
        } catch (PDOException $e) {
            return $this->responder->fail('500', '查询字典失败');
        }

        $list = [];
        foreach ($rows as $row) {
            $item = [
                'id'               => (int)$row['id'],
                'name'             => (string)$row['name'],
                'code'             => (string)$row['code'],
                'description'      => (string)$row['description'],
                'isSystem'         => (bool)$row['is_system'],
                'createUserString' => (string)$row['create_user_string'],
                'createTime'       => (string)$row['create_time'],
                'updateUserString' => (string)$row['update_user_string'],
                'updateTime'       => $row['update_time'] ? (string)$row['update_time'] : '',
            ];
            if ($desc !== '' &&
                !str_contains($item['name'], $desc) &&
                !str_contains($item['description'], $desc)
            ) {
                continue;
            }
            $list[] = $item;
        }

        return $this->responder->ok($list);
    }

    /**
     * 分页查询字典项：GET /system/dict/item。
     *
     * query:
     * - page, size
     * - description
     * - status
     * - dictId
     */
    private function listDictItem(ServerRequestInterface $request): ResponseInterface
    {
        $params = $request->getQueryParams();
        $page = max(1, (int)($params['page'] ?? 1));
        $size = max(1, (int)($params['size'] ?? 10));
        $description = trim((string)($params['description'] ?? ''));
        $statusStr = trim((string)($params['status'] ?? ''));
        $status = $statusStr !== '' ? (int)$statusStr : 0;
        $dictIdStr = trim((string)($params['dictId'] ?? ''));

        $baseSql = "
SELECT di.id,
       di.label,
       di.value,
       COALESCE(di.color, '')        AS color,
       COALESCE(di.sort, 999)        AS sort,
       COALESCE(di.description, '')  AS description,
       di.status,
       di.dict_id,
       di.create_time,
       COALESCE(cu.nickname, '')     AS create_user_string,
       di.update_time,
       COALESCE(uu.nickname, '')     AS update_user_string
FROM sys_dict_item AS di
LEFT JOIN sys_user AS cu ON cu.id = di.create_user
LEFT JOIN sys_user AS uu ON uu.id = di.update_user";

        $orderBy = ' ORDER BY di.sort ASC, di.id ASC';

        try {
            if ($dictIdStr !== '') {
                $dictId = (int)$dictIdStr;
                if ($dictId <= 0) {
                    return $this->responder->fail('400', '字典 ID 不正确');
                }
                $stmt = $this->db->prepare($baseSql . ' WHERE di.dict_id = :dictId' . $orderBy);
                $stmt->execute([':dictId' => $dictId]);
            } else {
                $stmt = $this->db->query($baseSql . $orderBy);
            }
            $rows = $stmt->fetchAll();
        } catch (PDOException $e) {
            return $this->responder->fail('500', '查询字典项失败');
        }

        $filtered = [];
        foreach ($rows as $row) {
            $item = [
                'id'               => (int)$row['id'],
                'label'            => (string)$row['label'],
                'value'            => (string)$row['value'],
                'color'            => (string)$row['color'],
                'sort'             => (int)$row['sort'],
                'description'      => (string)$row['description'],
                'status'           => (int)$row['status'],
                'dictId'           => (int)$row['dict_id'],
                'createUserString' => (string)$row['create_user_string'],
                'createTime'       => (string)$row['create_time'],
                'updateUserString' => (string)$row['update_user_string'],
                'updateTime'       => $row['update_time'] ? (string)$row['update_time'] : '',
            ];
            if ($description !== '' &&
                !str_contains($item['label'], $description) &&
                !str_contains($item['description'], $description)
            ) {
                continue;
            }
            if ($status !== 0 && $item['status'] !== $status) {
                continue;
            }
            $filtered[] = $item;
        }

        $total = count($filtered);
        $start = ($page - 1) * $size;
        if ($start > $total) {
            $start = $total;
        }
        $end = min($start + $size, $total);
        $pageList = array_slice($filtered, $start, $end - $start);

        return $this->responder->ok([
            'list'  => $pageList,
            'total' => $total,
        ]);
    }

    /**
     * 查询字典项详情：GET /system/dict/item/{id}。
     */
    private function getDictItem(ServerRequestInterface $request, array $args): ResponseInterface
    {
        $id = (int)($args['id'] ?? 0);
        if ($id <= 0) {
            return $this->responder->fail('400', 'ID 参数不正确');
        }

        $sql = "
SELECT di.id,
       di.label,
       di.value,
       COALESCE(di.color, '')        AS color,
       COALESCE(di.sort, 999)        AS sort,
       COALESCE(di.description, '')  AS description,
       di.status,
       di.dict_id,
       di.create_time,
       COALESCE(cu.nickname, '')     AS create_user_string,
       di.update_time,
       COALESCE(uu.nickname, '')     AS update_user_string
FROM sys_dict_item AS di
LEFT JOIN sys_user AS cu ON cu.id = di.create_user
LEFT JOIN sys_user AS uu ON uu.id = di.update_user
WHERE di.id = :id";

        try {
            $stmt = $this->db->prepare($sql);
            $stmt->execute([':id' => $id]);
            $row = $stmt->fetch();
        } catch (PDOException $e) {
            return $this->responder->fail('500', '查询字典项失败');
        }

        if ($row === false) {
            return $this->responder->fail('404', '字典项不存在');
        }

        $resp = [
            'id'               => (int)$row['id'],
            'label'            => (string)$row['label'],
            'value'            => (string)$row['value'],
            'color'            => (string)$row['color'],
            'sort'             => (int)$row['sort'],
            'description'      => (string)$row['description'],
            'status'           => (int)$row['status'],
            'dictId'           => (int)$row['dict_id'],
            'createUserString' => (string)$row['create_user_string'],
            'createTime'       => (string)$row['create_time'],
            'updateUserString' => (string)$row['update_user_string'],
            'updateTime'       => $row['update_time'] ? (string)$row['update_time'] : '',
        ];

        return $this->responder->ok($resp);
    }

    /**
     * 新增字典项：POST /system/dict/item。
     */
    private function createDictItem(ServerRequestInterface $request): ResponseInterface
    {
        try {
            $userId = $this->currentUserId($request);
        } catch (\RuntimeException $e) {
            return $this->responder->fail('401', $e->getMessage());
        }

        $body = json_decode((string)$request->getBody(), true);
        if (!is_array($body)) {
            return $this->responder->fail('400', '请求参数不正确');
        }

        $label = trim((string)($body['label'] ?? ''));
        $value = trim((string)($body['value'] ?? ''));
        $dictId = (int)($body['dictId'] ?? 0);
        if ($label === '' || $value === '' || $dictId <= 0) {
            return $this->responder->fail('400', '标签、值和字典 ID 不能为空');
        }
        $color = (string)($body['color'] ?? '');
        $sort = (int)($body['sort'] ?? 999);
        if ($sort <= 0) {
            $sort = 999;
        }
        $description = (string)($body['description'] ?? '');
        $status = (int)($body['status'] ?? 1);
        if ($status <= 0) {
            $status = 1;
        }

        // 校验同一字典下 value 是否重复
        $checkSql = "
SELECT 1
FROM sys_dict_item
WHERE dict_id = :dict_id AND value = :value
LIMIT 1";

        try {
            $stmt = $this->db->prepare($checkSql);
            $stmt->execute([
                ':dict_id' => $dictId,
                ':value'   => $value,
            ]);
            if ($stmt->fetch()) {
                return $this->responder->fail('400', '新增失败，字典值 [' . $value . '] 已存在');
            }
        } catch (PDOException $e) {
            return $this->responder->fail('500', '新增字典项失败');
        }

        $now = (new \DateTimeImmutable())->format('Y-m-d H:i:s');
        $newId = $this->nextId();

        $sql = "
INSERT INTO sys_dict_item (
    id, label, value, color, sort, description, status, dict_id,
    create_user, create_time
) VALUES (
    :id, :label, :value, :color, :sort, :description, :status, :dict_id,
    :create_user, :create_time
)";

        try {
            $stmt = $this->db->prepare($sql);
            $stmt->execute([
                ':id'          => $newId,
                ':label'       => $label,
                ':value'       => $value,
                ':color'       => $color,
                ':sort'        => $sort,
                ':description' => $description,
                ':status'      => $status,
                ':dict_id'     => $dictId,
                ':create_user' => $userId,
                ':create_time' => $now,
            ]);
        } catch (PDOException $e) {
            return $this->responder->fail('500', '新增字典项失败');
        }

        return $this->responder->ok(true);
    }

    /**
     * 修改字典项：PUT /system/dict/item/{id}。
     */
    private function updateDictItem(ServerRequestInterface $request, array $args): ResponseInterface
    {
        try {
            $userId = $this->currentUserId($request);
        } catch (\RuntimeException $e) {
            return $this->responder->fail('401', $e->getMessage());
        }

        $id = (int)($args['id'] ?? 0);
        if ($id <= 0) {
            return $this->responder->fail('400', 'ID 参数不正确');
        }

        $body = json_decode((string)$request->getBody(), true);
        if (!is_array($body)) {
            return $this->responder->fail('400', '请求参数不正确');
        }

        $label = trim((string)($body['label'] ?? ''));
        $value = trim((string)($body['value'] ?? ''));
        if ($label === '' || $value === '') {
            return $this->responder->fail('400', '标签和值不能为空');
        }
        $color = (string)($body['color'] ?? '');
        $sort = (int)($body['sort'] ?? 999);
        if ($sort <= 0) {
            $sort = 999;
        }
        $description = (string)($body['description'] ?? '');
        $status = (int)($body['status'] ?? 1);
        if ($status <= 0) {
            $status = 1;
        }

        // 查询当前字典项所属字典
        $dictSql = 'SELECT dict_id FROM sys_dict_item WHERE id = :id';
        try {
            $stmt = $this->db->prepare($dictSql);
            $stmt->execute([':id' => $id]);
            $row = $stmt->fetch();
        } catch (PDOException $e) {
            return $this->responder->fail('500', '修改字典项失败');
        }
        if ($row === false) {
            return $this->responder->fail('404', '字典项不存在');
        }
        $dictId = (int)$row['dict_id'];

        // 校验同一字典下 value 是否重复（排除自身）
        $checkSql = "
SELECT 1
FROM sys_dict_item
WHERE dict_id = :dict_id AND value = :value AND id <> :id
LIMIT 1";
        try {
            $stmt = $this->db->prepare($checkSql);
            $stmt->execute([
                ':dict_id' => $dictId,
                ':value'   => $value,
                ':id'      => $id,
            ]);
            if ($stmt->fetch()) {
                return $this->responder->fail('400', '修改失败，字典值 [' . $value . '] 已存在');
            }
        } catch (PDOException $e) {
            return $this->responder->fail('500', '修改字典项失败');
        }

        $now = (new \DateTimeImmutable())->format('Y-m-d H:i:s');

        $sql = "
UPDATE sys_dict_item
   SET label       = :label,
       value       = :value,
       color       = :color,
       sort        = :sort,
       description = :description,
       status      = :status,
       update_user = :update_user,
       update_time = :update_time
 WHERE id          = :id";

        try {
            $stmt = $this->db->prepare($sql);
            $stmt->execute([
                ':label'       => $label,
                ':value'       => $value,
                ':color'       => $color,
                ':sort'        => $sort,
                ':description' => $description,
                ':status'      => $status,
                ':update_user' => $userId,
                ':update_time' => $now,
                ':id'          => $id,
            ]);
        } catch (PDOException $e) {
            return $this->responder->fail('500', '修改字典项失败');
        }

        return $this->responder->ok(true);
    }

    /**
     * 删除字典项：DELETE /system/dict/item。
     *
     * body: { ids: number[] }
     */
    private function deleteDictItem(ServerRequestInterface $request): ResponseInterface
    {
        try {
            $this->currentUserId($request);
        } catch (\RuntimeException $e) {
            return $this->responder->fail('401', $e->getMessage());
        }

        $body = json_decode((string)$request->getBody(), true);
        $ids = is_array($body['ids'] ?? null) ? $body['ids'] : [];
        $ids = array_map('intval', $ids);
        $ids = array_values(array_filter($ids, fn ($v) => $v > 0));
        if (!$ids) {
            return $this->responder->fail('400', 'ID 列表不能为空');
        }

        $in = implode(',', array_fill(0, count($ids), '?'));
        $sql = "DELETE FROM sys_dict_item WHERE id IN ($in)";

        try {
            $stmt = $this->db->prepare($sql);
            $stmt->execute($ids);
        } catch (PDOException $e) {
            return $this->responder->fail('500', '删除字典项失败');
        }

        return $this->responder->ok(true);
    }

    /**
     * 查询单个字典：GET /system/dict/{id}。
     */
    private function getDict(ServerRequestInterface $request, array $args): ResponseInterface
    {
        $id = (int)($args['id'] ?? 0);
        if ($id <= 0) {
            return $this->responder->fail('400', 'ID 参数不正确');
        }

        $sql = "
SELECT d.id,
       d.name,
       d.code,
       COALESCE(d.description, '')         AS description,
       COALESCE(d.is_system, FALSE)        AS is_system,
       d.create_time,
       COALESCE(cu.nickname, '')           AS create_user_string,
       d.update_time,
       COALESCE(uu.nickname, '')           AS update_user_string
FROM sys_dict AS d
LEFT JOIN sys_user AS cu ON cu.id = d.create_user
LEFT JOIN sys_user AS uu ON uu.id = d.update_user
WHERE d.id = :id";

        try {
            $stmt = $this->db->prepare($sql);
            $stmt->execute([':id' => $id]);
            $row = $stmt->fetch();
        } catch (PDOException $e) {
            return $this->responder->fail('500', '查询字典失败');
        }

        if ($row === false) {
            return $this->responder->fail('404', '字典不存在');
        }

        $resp = [
            'id'               => (int)$row['id'],
            'name'             => (string)$row['name'],
            'code'             => (string)$row['code'],
            'description'      => (string)$row['description'],
            'isSystem'         => (bool)$row['is_system'],
            'createUserString' => (string)$row['create_user_string'],
            'createTime'       => (string)$row['create_time'],
            'updateUserString' => (string)$row['update_user_string'],
            'updateTime'       => $row['update_time'] ? (string)$row['update_time'] : '',
        ];

        return $this->responder->ok($resp);
    }

    /**
     * 新增字典：POST /system/dict。
     */
    private function createDict(ServerRequestInterface $request): ResponseInterface
    {
        try {
            $userId = $this->currentUserId($request);
        } catch (\RuntimeException $e) {
            return $this->responder->fail('401', $e->getMessage());
        }

        $body = json_decode((string)$request->getBody(), true);
        if (!is_array($body)) {
            return $this->responder->fail('400', '请求参数不正确');
        }

        $name = trim((string)($body['name'] ?? ''));
        $code = trim((string)($body['code'] ?? ''));
        if ($name === '' || $code === '') {
            return $this->responder->fail('400', '名称和编码不能为空');
        }
        $description = (string)($body['description'] ?? '');

        // 唯一性校验
        try {
            $stmt = $this->db->prepare('SELECT 1 FROM sys_dict WHERE name = :name LIMIT 1');
            $stmt->execute([':name' => $name]);
            if ($stmt->fetch()) {
                return $this->responder->fail('400', '新增失败，[' . $name . '] 已存在');
            }
            $stmt = $this->db->prepare('SELECT 1 FROM sys_dict WHERE code = :code LIMIT 1');
            $stmt->execute([':code' => $code]);
            if ($stmt->fetch()) {
                return $this->responder->fail('400', '新增失败，[' . $code . '] 已存在');
            }
        } catch (PDOException $e) {
            return $this->responder->fail('500', '新增字典失败');
        }

        $now = (new \DateTimeImmutable())->format('Y-m-d H:i:s');
        $newId = $this->nextId();

        $sql = "
INSERT INTO sys_dict (
    id, name, code, description, is_system, create_user, create_time
) VALUES (
    :id, :name, :code, :description, FALSE, :create_user, :create_time
)";

        try {
            $stmt = $this->db->prepare($sql);
            $stmt->execute([
                ':id'          => $newId,
                ':name'        => $name,
                ':code'        => $code,
                ':description' => $description,
                ':create_user' => $userId,
                ':create_time' => $now,
            ]);
        } catch (PDOException $e) {
            return $this->responder->fail('500', '新增字典失败');
        }

        return $this->responder->ok(['id' => $newId]);
    }

    /**
     * 修改字典：PUT /system/dict/{id}。
     */
    private function updateDict(ServerRequestInterface $request, array $args): ResponseInterface
    {
        try {
            $userId = $this->currentUserId($request);
        } catch (\RuntimeException $e) {
            return $this->responder->fail('401', $e->getMessage());
        }

        $id = (int)($args['id'] ?? 0);
        if ($id <= 0) {
            return $this->responder->fail('400', 'ID 参数不正确');
        }

        $body = json_decode((string)$request->getBody(), true);
        if (!is_array($body)) {
            return $this->responder->fail('400', '请求参数不正确');
        }

        $name = trim((string)($body['name'] ?? ''));
        if ($name === '') {
            return $this->responder->fail('400', '名称不能为空');
        }
        $description = (string)($body['description'] ?? '');

        $now = (new \DateTimeImmutable())->format('Y-m-d H:i:s');

        $sql = "
UPDATE sys_dict
   SET name        = :name,
       description = :description,
       update_user = :update_user,
       update_time = :update_time
 WHERE id          = :id";

        try {
            $stmt = $this->db->prepare($sql);
            $stmt->execute([
                ':name'        => $name,
                ':description' => $description,
                ':update_user' => $userId,
                ':update_time' => $now,
                ':id'          => $id,
            ]);
        } catch (PDOException $e) {
            return $this->responder->fail('500', '修改字典失败');
        }

        return $this->responder->ok(true);
    }

    /**
     * 删除字典：DELETE /system/dict。
     *
     * body: { ids: number[] }
     */
    private function deleteDict(ServerRequestInterface $request): ResponseInterface
    {
        try {
            $this->currentUserId($request);
        } catch (\RuntimeException $e) {
            return $this->responder->fail('401', $e->getMessage());
        }

        $body = json_decode((string)$request->getBody(), true);
        $ids = is_array($body['ids'] ?? null) ? $body['ids'] : [];
        $ids = array_map('intval', $ids);
        $ids = array_values(array_filter($ids, fn ($v) => $v > 0));
        if (!$ids) {
            return $this->responder->fail('400', 'ID 列表不能为空');
        }

        // 校验系统内置字典
        try {
            $in = implode(',', array_fill(0, count($ids), '?'));
            $sql = "SELECT name, COALESCE(is_system, FALSE) AS is_system FROM sys_dict WHERE id IN ($in)";
            $stmt = $this->db->prepare($sql);
            $stmt->execute($ids);
            $rows = $stmt->fetchAll();
        } catch (PDOException $e) {
            return $this->responder->fail('500', '删除字典失败');
        }

        foreach ($rows as $row) {
            if ((bool)$row['is_system']) {
                return $this->responder->fail(
                    '400',
                    '所选字典 [' . $row['name'] . '] 是系统内置字典，不允许删除'
                );
            }
        }

        try {
            $this->db->beginTransaction();

            $in = implode(',', array_fill(0, count($ids), '?'));
            $stmt = $this->db->prepare("DELETE FROM sys_dict_item WHERE dict_id IN ($in)");
            $stmt->execute($ids);

            $stmt = $this->db->prepare("DELETE FROM sys_dict WHERE id IN ($in)");
            $stmt->execute($ids);

            $this->db->commit();
        } catch (PDOException $e) {
            $this->db->rollBack();
            return $this->responder->fail('500', '删除字典失败');
        }

        return $this->responder->ok(true);
    }

    /**
     * 清理字典缓存：DELETE /system/dict/cache/{code}。
     *
     * 当前 PHP 版本未实现缓存，直接返回成功以兼容前端。
     */
    private function clearDictCache(ServerRequestInterface $request, array $args): ResponseInterface
    {
        return $this->responder->ok(true);
    }

    /**
     * 简单 ID 生成器，保持与其他路由一致。
     */
    private function nextId(): int
    {
        return (int)(microtime(true) * 1000000) + random_int(1, 1000);
    }
}

