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
 * 客户端配置路由：/system/client 系列接口。
 */
final class ClientRoutes
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

    public function register(App $app): void
    {
        $app->get('/system/client', fn (ServerRequestInterface $r) => $this->listClientPage($r));
        $app->get('/system/client/{id}', fn (ServerRequestInterface $r, array $a) => $this->getClient($r, $a));
        $app->post('/system/client', fn (ServerRequestInterface $r) => $this->createClient($r));
        $app->put('/system/client/{id}', fn (ServerRequestInterface $r, array $a) => $this->updateClient($r, $a));
        $app->delete('/system/client', fn (ServerRequestInterface $r) => $this->deleteClient($r));
    }

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

    private function listClientPage(ServerRequestInterface $request): ResponseInterface
    {
        $params = $request->getQueryParams();
        $page = max(1, (int)($params['page'] ?? 1));
        $size = max(1, (int)($params['size'] ?? 10));

        $clientType = trim((string)($params['clientType'] ?? ''));
        $statusStr = trim((string)($params['status'] ?? ''));
        $status = $statusStr !== '' ? (int)$statusStr : 0;
        $authTypes = $params['authType'] ?? [];
        if (!is_array($authTypes)) {
            $authTypes = [$authTypes];
        }

        $where = 'WHERE 1=1';
        $args = [];

        if ($clientType !== '') {
            $where .= ' AND c.client_type = :client_type';
            $args[':client_type'] = $clientType;
        }
        if ($status !== 0) {
            $where .= ' AND c.status = :status';
            $args[':status'] = $status;
        }
        if ($authTypes) {
            $conds = [];
            foreach ($authTypes as $idx => $t) {
                $key = ':auth' . $idx;
                $conds[] = "c.auth_type::text ILIKE {$key}";
                $args[$key] = '%' . $t . '%';
            }
            if ($conds) {
                $where .= ' AND (' . implode(' OR ', $conds) . ')';
            }
        }

        $countSql = 'SELECT COUNT(*) FROM sys_client AS c ' . $where;
        try {
            $stmt = $this->db->prepare($countSql);
            $stmt->execute($args);
            $total = (int)$stmt->fetchColumn();
        } catch (PDOException $e) {
            return $this->responder->fail('500', '查询客户端失败');
        }

        if ($total === 0) {
            return $this->responder->ok(['list' => [], 'total' => 0]);
        }

        $offset = ($page - 1) * $size;
        $sql = "
SELECT c.id,
       c.client_id,
       c.client_type,
       c.auth_type,
       c.active_timeout,
       c.timeout,
       c.status,
       c.create_user,
       c.create_time,
       c.update_user,
       c.update_time,
       COALESCE(cu.nickname, ''),
       COALESCE(uu.nickname, '')
FROM sys_client AS c
LEFT JOIN sys_user AS cu ON cu.id = c.create_user
LEFT JOIN sys_user AS uu ON uu.id = c.update_user
{$where}
ORDER BY c.id DESC
LIMIT :limit OFFSET :offset";

        try {
            $stmt = $this->db->prepare($sql);
            foreach ($args as $k => $v) {
                $stmt->bindValue($k, $v);
            }
            $stmt->bindValue(':limit', $size, PDO::PARAM_INT);
            $stmt->bindValue(':offset', $offset, PDO::PARAM_INT);
            $stmt->execute();
        } catch (PDOException $e) {
            return $this->responder->fail('500', '查询客户端失败');
        }

        $list = [];
        while ($row = $stmt->fetch(PDO::FETCH_ASSOC)) {
            $authType = [];
            if (isset($row['auth_type']) && $row['auth_type'] !== null) {
                $decoded = json_decode((string)$row['auth_type'], true);
                if (is_array($decoded)) {
                    $authType = array_values(array_map('strval', $decoded));
                }
            }
            $nickCreate = (string)$row['nickname'];
            $nickUpdate = (string)$row['nickname'];

            $item = [
                'id'               => (int)$row['id'],
                'clientId'         => (string)$row['client_id'],
                'clientType'       => (string)$row['client_type'],
                'authType'         => $authType,
                'activeTimeout'    => (int)$row['active_timeout'],
                'timeout'          => (int)$row['timeout'],
                'status'           => (int)$row['status'],
                'createUser'       => $nickCreate,
                'createTime'       => (string)$row['create_time'],
                'updateUser'       => $nickUpdate,
                'updateTime'       => $row['update_time'] ? (string)$row['update_time'] : '',
                'createUserString' => $nickCreate,
                'updateUserString' => $nickUpdate,
            ];
            $list[] = $item;
        }

        return $this->responder->ok(['list' => $list, 'total' => $total]);
    }

    private function getClient(ServerRequestInterface $request, array $args): ResponseInterface
    {
        $id = (int)($args['id'] ?? 0);
        if ($id <= 0) {
            return $this->responder->fail('400', 'ID 参数不正确');
        }

        $sql = "
SELECT c.id,
       c.client_id,
       c.client_type,
       c.auth_type,
       c.active_timeout,
       c.timeout,
       c.status,
       c.create_user,
       c.create_time,
       c.update_user,
       c.update_time,
       COALESCE(cu.nickname, ''),
       COALESCE(uu.nickname, '')
FROM sys_client AS c
LEFT JOIN sys_user AS cu ON cu.id = c.create_user
LEFT JOIN sys_user AS uu ON uu.id = c.update_user
WHERE c.id = :id";

        $stmt = $this->db->prepare($sql);
        $stmt->execute([':id' => $id]);
        $row = $stmt->fetch(PDO::FETCH_ASSOC);
        if ($row === false) {
            return $this->responder->fail('404', '客户端不存在');
        }

        $authType = [];
        if (isset($row['auth_type']) && $row['auth_type'] !== null) {
            $decoded = json_decode((string)$row['auth_type'], true);
            if (is_array($decoded)) {
                $authType = array_values(array_map('strval', $decoded));
            }
        }
        $nickCreate = (string)$row['nickname'];
        $nickUpdate = (string)$row['nickname'];

        $resp = [
            'id'               => (int)$row['id'],
            'clientId'         => (string)$row['client_id'],
            'clientType'       => (string)$row['client_type'],
            'authType'         => $authType,
            'activeTimeout'    => (int)$row['active_timeout'],
            'timeout'          => (int)$row['timeout'],
            'status'           => (int)$row['status'],
            'createUser'       => $nickCreate,
            'createTime'       => (string)$row['create_time'],
            'updateUser'       => $nickUpdate,
            'updateTime'       => $row['update_time'] ? (string)$row['update_time'] : '',
            'createUserString' => $nickCreate,
            'updateUserString' => $nickUpdate,
        ];

        return $this->responder->ok($resp);
    }

    private function createClient(ServerRequestInterface $request): ResponseInterface
    {
        try {
            $userId = $this->currentUserId($request);
        } catch (\RuntimeException $e) {
            return $this->responder->fail('401', $e->getMessage());
        }

        $data = json_decode((string)$request->getBody(), true);
        if (!is_array($data)) {
            return $this->responder->fail('400', '请求参数不正确');
        }

        $clientType = trim((string)($data['clientType'] ?? ''));
        $authType = $data['authType'] ?? [];
        if (!is_array($authType)) {
            $authType = [];
        }
        $activeTimeout = (int)($data['activeTimeout'] ?? 1800);
        $timeout = (int)($data['timeout'] ?? 86400);
        $status = (int)($data['status'] ?? 1);

        if ($clientType === '' || !$authType) {
            return $this->responder->fail('400', '客户端类型和认证类型不能为空');
        }
        if ($activeTimeout === 0) {
            $activeTimeout = 1800;
        }
        if ($timeout === 0) {
            $timeout = 86400;
        }
        if ($status === 0) {
            $status = 1;
        }

        $clientId = dechex($this->nextId());
        $authJson = json_encode(array_values(array_map('strval', $authType)), JSON_UNESCAPED_UNICODE);
        if ($authJson === false) {
            return $this->responder->fail('500', '保存客户端失败');
        }

        $now = (new \DateTimeImmutable())->format('Y-m-d H:i:s');
        $newId = $this->nextId();

        $sql = "
INSERT INTO sys_client (
    id, client_id, client_type, auth_type,
    active_timeout, timeout, status,
    create_user, create_time
) VALUES (
    :id, :client_id, :client_type, :auth_type,
    :active_timeout, :timeout, :status,
    :create_user, :create_time
)";

        try {
            $stmt = $this->db->prepare($sql);
            $stmt->execute([
                ':id'             => $newId,
                ':client_id'      => $clientId,
                ':client_type'    => $clientType,
                ':auth_type'      => $authJson,
                ':active_timeout' => $activeTimeout,
                ':timeout'        => $timeout,
                ':status'         => $status,
                ':create_user'    => $userId,
                ':create_time'    => $now,
            ]);
        } catch (PDOException $e) {
            return $this->responder->fail('500', '新增客户端失败');
        }

        return $this->responder->ok(['id' => $newId]);
    }

    private function updateClient(ServerRequestInterface $request, array $args): ResponseInterface
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

        $data = json_decode((string)$request->getBody(), true);
        if (!is_array($data)) {
            return $this->responder->fail('400', '请求参数不正确');
        }

        $clientType = trim((string)($data['clientType'] ?? ''));
        $authType = $data['authType'] ?? [];
        if (!is_array($authType)) {
            $authType = [];
        }
        $activeTimeout = (int)($data['activeTimeout'] ?? 1800);
        $timeout = (int)($data['timeout'] ?? 86400);
        $status = (int)($data['status'] ?? 1);

        if ($clientType === '' || !$authType) {
            return $this->responder->fail('400', '客户端类型和认证类型不能为空');
        }
        if ($status === 0) {
            $status = 1;
        }

        $authJson = json_encode(array_values(array_map('strval', $authType)), JSON_UNESCAPED_UNICODE);
        if ($authJson === false) {
            return $this->responder->fail('500', '保存客户端失败');
        }

        $now = (new \DateTimeImmutable())->format('Y-m-d H:i:s');

        $sql = "
UPDATE sys_client
   SET client_type    = :client_type,
       auth_type      = :auth_type,
       active_timeout = :active_timeout,
       timeout        = :timeout,
       status         = :status,
       update_user    = :update_user,
       update_time    = :update_time
 WHERE id             = :id";

        try {
            $stmt = $this->db->prepare($sql);
            $stmt->execute([
                ':client_type'    => $clientType,
                ':auth_type'      => $authJson,
                ':active_timeout' => $activeTimeout,
                ':timeout'        => $timeout,
                ':status'         => $status,
                ':update_user'    => $userId,
                ':update_time'    => $now,
                ':id'             => $id,
            ]);
        } catch (PDOException $e) {
            return $this->responder->fail('500', '修改客户端失败');
        }

        return $this->responder->ok(true);
    }

    private function deleteClient(ServerRequestInterface $request): ResponseInterface
    {
        try {
            $this->currentUserId($request);
        } catch (\RuntimeException $e) {
            return $this->responder->fail('401', $e->getMessage());
        }

        $data = json_decode((string)$request->getBody(), true);
        $ids = is_array($data['ids'] ?? null) ? $data['ids'] : [];
        $ids = array_map('intval', $ids);
        $ids = array_values(array_filter($ids, fn ($v) => $v > 0));
        if (!$ids) {
            return $this->responder->fail('400', 'ID 列表不能为空');
        }

        try {
            $this->db->beginTransaction();
            $in = implode(',', array_fill(0, count($ids), '?'));
            $sql = "DELETE FROM sys_client WHERE id IN ($in)";
            $stmt = $this->db->prepare($sql);
            $stmt->execute($ids);
            $this->db->commit();
        } catch (PDOException $e) {
            $this->db->rollBack();
            return $this->responder->fail('500', '删除客户端失败');
        }

        return $this->responder->ok(true);
    }

    private function nextId(): int
    {
        return (int)(microtime(true) * 1000000) + random_int(1, 1000);
    }
}

