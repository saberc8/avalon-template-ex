<?php

declare(strict_types=1);

namespace Voc\Admin\Interfaces\Http;

use PDO;
use Psr\Http\Message\ResponseInterface;
use Psr\Http\Message\ServerRequestInterface;
use Slim\App;
use Voc\Admin\Infrastructure\Response\JsonResponder;
use Voc\Admin\Infrastructure\Security\TokenService;

/**
 * 部门管理路由：/system/dept 系列接口。
 */
final class DeptRoutes
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
        $app->get('/system/dept/tree', fn (ServerRequestInterface $r) => $this->listDeptTree($r));
        $app->get('/system/dept/{id}', fn (ServerRequestInterface $r, array $a) => $this->getDept($r, $a));
        $app->post('/system/dept', fn (ServerRequestInterface $r) => $this->createDept($r));
        $app->put('/system/dept/{id}', fn (ServerRequestInterface $r, array $a) => $this->updateDept($r, $a));
        $app->delete('/system/dept', fn (ServerRequestInterface $r) => $this->deleteDept($r));
        $app->get('/system/dept/export', fn (ServerRequestInterface $r) => $this->exportDept($r));
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

    private function listDeptTree(ServerRequestInterface $request): ResponseInterface
    {
        $params = $request->getQueryParams();
        $desc = trim((string)($params['description'] ?? ''));
        $statusStr = trim((string)($params['status'] ?? ''));

        $where = 'WHERE 1=1';
        $args = [];
        if ($desc !== '') {
            $where .= ' AND (d.name ILIKE :desc OR COALESCE(d.description,\'\') ILIKE :desc)';
            $args[':desc'] = '%' . $desc . '%';
        }
        if ($statusStr !== '') {
            $where .= ' AND d.status = :status';
            $args[':status'] = (int)$statusStr;
        }

        $sql = "
SELECT d.id,
       d.name,
       d.parent_id,
       d.sort,
       d.status,
       d.is_system,
       COALESCE(d.description, ''),
       d.create_time,
       COALESCE(cu.nickname, ''),
       d.update_time,
       COALESCE(uu.nickname, '')
FROM sys_dept AS d
LEFT JOIN sys_user AS cu ON cu.id = d.create_user
LEFT JOIN sys_user AS uu ON uu.id = d.update_user
{$where}
ORDER BY d.sort ASC, d.id ASC";
        $stmt = $this->db->prepare($sql);
        $stmt->execute($args);
        $rows = $stmt->fetchAll();

        if (!$rows) {
            return $this->responder->ok([]);
        }

        $nodeMap = [];
        foreach ($rows as $row) {
            $nodeMap[(int)$row['id']] = [
                'id'               => (int)$row['id'],
                'name'             => (string)$row['name'],
                'parentId'         => (int)$row['parent_id'],
                'sort'             => (int)$row['sort'],
                'status'           => (int)$row['status'],
                'isSystem'         => (bool)$row['is_system'],
                'description'      => (string)$row['description'],
                'createUserString' => (string)$row['nickname'],
                'createTime'       => (string)$row['create_time'],
                'updateUserString' => (string)$row['nickname'],
                'updateTime'       => (string)$row['update_time'],
                'children'         => [],
            ];
        }

        $roots = [];
        foreach ($nodeMap as $id => &$node) {
            $pid = $node['parentId'];
            if ($pid === 0 || !isset($nodeMap[$pid])) {
                $roots[] = &$node;
                continue;
            }
            $nodeMap[$pid]['children'][] = $node;
        }
        unset($node);

        return $this->responder->ok(array_values($roots));
    }

    private function getDept(ServerRequestInterface $request, array $args): ResponseInterface
    {
        $id = (int)($args['id'] ?? 0);
        if ($id <= 0) {
            return $this->responder->fail('400', '无效的部门 ID');
        }

        $sql = "
SELECT d.id,
       d.name,
       d.parent_id,
       d.sort,
       d.status,
       d.is_system,
       COALESCE(d.description, ''),
       d.create_time,
       COALESCE(cu.nickname, ''),
       d.update_time,
       COALESCE(uu.nickname, '')
FROM sys_dept AS d
LEFT JOIN sys_user AS cu ON cu.id = d.create_user
LEFT JOIN sys_user AS uu ON uu.id = d.update_user
WHERE d.id = :id";
        $stmt = $this->db->prepare($sql);
        $stmt->execute([':id' => $id]);
        $row = $stmt->fetch();
        if ($row === false) {
            return $this->responder->fail('404', '部门不存在');
        }

        $resp = [
            'id'               => (int)$row['id'],
            'name'             => (string)$row['name'],
            'parentId'         => (int)$row['parent_id'],
            'sort'             => (int)$row['sort'],
            'status'           => (int)$row['status'],
            'isSystem'         => (bool)$row['is_system'],
            'description'      => (string)$row['description'],
            'createUserString' => (string)$row['nickname'],
            'createTime'       => (string)$row['create_time'],
            'updateUserString' => (string)$row['nickname'],
            'updateTime'       => (string)$row['update_time'],
            'children'         => [],
        ];

        return $this->responder->ok($resp);
    }

    private function createDept(ServerRequestInterface $request): ResponseInterface
    {
        $data = json_decode((string)$request->getBody(), true);
        if (!is_array($data)) {
            return $this->responder->fail('400', '参数错误');
        }

        $name = trim((string)($data['name'] ?? ''));
        $parentId = (int)($data['parentId'] ?? 0);
        $sort = (int)($data['sort'] ?? 1);
        $status = (int)($data['status'] ?? 1);
        $desc = (string)($data['description'] ?? '');

        if ($name === '') {
            return $this->responder->fail('400', '名称不能为空');
        }
        if ($parentId === 0) {
            return $this->responder->fail('400', '上级部门不能为空');
        }
        if ($sort <= 0) {
            $sort = 1;
        }
        if ($status === 0) {
            $status = 1;
        }

        $stmt = $this->db->prepare('SELECT 1 FROM sys_dept WHERE name = :name AND parent_id = :pid LIMIT 1');
        $stmt->execute([':name' => $name, ':pid' => $parentId]);
        if ($stmt->fetch()) {
            return $this->responder->fail('400', '新增失败，该名称在当前上级下已存在');
        }

        $stmt = $this->db->prepare('SELECT 1 FROM sys_dept WHERE id = :id');
        $stmt->execute([':id' => $parentId]);
        if (!$stmt->fetch()) {
            return $this->responder->fail('400', '上级部门不存在');
        }

        try {
            $userId = $this->currentUserId($request);
        } catch (\RuntimeException $e) {
            return $this->responder->fail('401', $e->getMessage());
        }

        $now = (new \DateTimeImmutable())->format('Y-m-d H:i:s');
        $newId = $this->nextId();

        $sql = "
INSERT INTO sys_dept (
    id, name, parent_id, sort, status, is_system, description,
    create_user, create_time
) VALUES (
    :id, :name, :parent_id, :sort, :status, FALSE, :description,
    :create_user, :create_time
)";
        $stmt = $this->db->prepare($sql);
        $stmt->execute([
            ':id'          => $newId,
            ':name'        => $name,
            ':parent_id'   => $parentId,
            ':sort'        => $sort,
            ':status'      => $status,
            ':description' => $desc,
            ':create_user' => $userId,
            ':create_time' => $now,
        ]);

        return $this->responder->ok(true);
    }

    private function updateDept(ServerRequestInterface $request, array $args): ResponseInterface
    {
        $id = (int)($args['id'] ?? 0);
        if ($id <= 0) {
            return $this->responder->fail('400', '无效的部门 ID');
        }

        $data = json_decode((string)$request->getBody(), true);
        if (!is_array($data)) {
            return $this->responder->fail('400', '参数错误');
        }

        $name = trim((string)($data['name'] ?? ''));
        $parentId = (int)($data['parentId'] ?? 0);
        $sort = (int)($data['sort'] ?? 1);
        $status = (int)($data['status'] ?? 1);
        $desc = (string)($data['description'] ?? '');

        if ($name === '') {
            return $this->responder->fail('400', '名称不能为空');
        }
        if ($parentId === 0) {
            return $this->responder->fail('400', '上级部门不能为空');
        }
        if ($sort <= 0) {
            $sort = 1;
        }
        if ($status === 0) {
            $status = 1;
        }

        $stmt = $this->db->prepare('SELECT id, name, parent_id, status, is_system FROM sys_dept WHERE id = :id');
        $stmt->execute([':id' => $id]);
        $row = $stmt->fetch();
        if ($row === false) {
            return $this->responder->fail('404', '部门不存在');
        }

        $oldName = (string)$row['name'];
        $oldParentId = (int)$row['parent_id'];
        $isSystem = (bool)$row['is_system'];

        if ($isSystem) {
            if ($status === 2) {
                return $this->responder->fail('400', '[' . $oldName . '] 是系统内置部门，不允许禁用');
            }
            if ($parentId !== $oldParentId) {
                return $this->responder->fail('400', '[' . $oldName . '] 是系统内置部门，不允许变更上级部门');
            }
        }

        $stmt = $this->db->prepare('SELECT 1 FROM sys_dept WHERE name = :name AND parent_id = :pid AND id <> :id LIMIT 1');
        $stmt->execute([':name' => $name, ':pid' => $parentId, ':id' => $id]);
        if ($stmt->fetch()) {
            return $this->responder->fail('400', '修改失败，该名称在当前上级下已存在');
        }

        try {
            $userId = $this->currentUserId($request);
        } catch (\RuntimeException $e) {
            return $this->responder->fail('401', $e->getMessage());
        }

        $now = (new \DateTimeImmutable())->format('Y-m-d H:i:s');

        $sql = "
UPDATE sys_dept
SET name = :name,
    parent_id = :parent_id,
    sort = :sort,
    status = :status,
    description = :description,
    update_user = :update_user,
    update_time = :update_time
WHERE id = :id";
        $stmt = $this->db->prepare($sql);
        $stmt->execute([
            ':name'        => $name,
            ':parent_id'   => $parentId,
            ':sort'        => $sort,
            ':status'      => $status,
            ':description' => $desc,
            ':update_user' => $userId,
            ':update_time' => $now,
            ':id'          => $id,
        ]);

        return $this->responder->ok(true);
    }

    private function deleteDept(ServerRequestInterface $request): ResponseInterface
    {
        $data = json_decode((string)$request->getBody(), true);
        $ids = is_array($data['ids'] ?? null) ? $data['ids'] : [];
        $ids = array_map('intval', $ids);
        $ids = array_values(array_filter($ids, fn ($v) => $v > 0));
        if (!$ids) {
            return $this->responder->fail('400', '参数错误');
        }

        $in = implode(',', array_fill(0, count($ids), '?'));

        $stmt = $this->db->prepare("SELECT name FROM sys_dept WHERE id IN ($in) AND is_system = TRUE LIMIT 1");
        $stmt->execute($ids);
        $row = $stmt->fetch();
        if ($row !== false) {
            return $this->responder->fail('400', '所选部门 [' . $row['name'] . '] 是系统内置部门，不允许删除');
        }

        $stmt = $this->db->prepare("SELECT 1 FROM sys_dept WHERE parent_id IN ($in) LIMIT 1");
        $stmt->execute($ids);
        if ($stmt->fetch()) {
            return $this->responder->fail('400', '所选部门存在下级部门，不允许删除');
        }

        $stmt = $this->db->prepare("SELECT 1 FROM sys_user WHERE dept_id IN ($in) LIMIT 1");
        $stmt->execute($ids);
        if ($stmt->fetch()) {
            return $this->responder->fail('400', '所选部门存在用户关联，请解除关联后重试');
        }

        $stmt = $this->db->prepare("DELETE FROM sys_role_dept WHERE dept_id IN ($in)");
        $stmt->execute($ids);

        $stmt = $this->db->prepare("DELETE FROM sys_dept WHERE id IN ($in)");
        $stmt->execute($ids);

        return $this->responder->ok(true);
    }

    private function exportDept(ServerRequestInterface $request): ResponseInterface
    {
        $params = $request->getQueryParams();
        $desc = trim((string)($params['description'] ?? ''));
        $statusStr = trim((string)($params['status'] ?? ''));

        $where = 'WHERE 1=1';
        $args = [];
        if ($desc !== '') {
            $where .= ' AND (d.name ILIKE :desc OR COALESCE(d.description,\'\') ILIKE :desc)';
            $args[':desc'] = '%' . $desc . '%';
        }
        if ($statusStr !== '') {
            $where .= ' AND d.status = :status';
            $args[':status'] = (int)$statusStr;
        }

        $sql = "
SELECT d.id,
       d.name,
       d.parent_id,
       d.status,
       d.sort,
       d.is_system,
       COALESCE(d.description, ''),
       d.create_time,
       COALESCE(cu.nickname, ''),
       d.update_time,
       COALESCE(uu.nickname, '')
FROM sys_dept AS d
LEFT JOIN sys_user AS cu ON cu.id = d.create_user
LEFT JOIN sys_user AS uu ON uu.id = d.update_user
{$where}
ORDER BY d.sort ASC, d.id ASC";
        $stmt = $this->db->prepare($sql);
        $stmt->execute($args);
        $rows = $stmt->fetchAll();

        $content = "ID,名称,上级部门ID,状态,排序,系统内置,描述,创建时间,创建人,修改时间,修改人\n";
        foreach ($rows as $row) {
            $ut = $row['update_time'] ? (string)$row['update_time'] : '';
            $content .= sprintf(
                "%d,%s,%d,%d,%d,%s,%s,%s,%s,%s,%s\n",
                $row['id'],
                $row['name'],
                $row['parent_id'],
                $row['status'],
                $row['sort'],
                $row['is_system'] ? 'true' : 'false',
                $row['description'],
                $row['create_time'],
                $row['nickname'],
                $ut,
                $row['nickname']
            );
        }

        $response = $this->responder->ok(null);
        $body = $response->getBody();
        $body->rewind();
        $body->write($content);
        return $response
            ->withHeader('Content-Type', 'text/csv; charset=utf-8')
            ->withHeader('Content-Disposition', 'attachment; filename="dept_export.csv"');
    }

    private function nextId(): int
    {
        return (int)(microtime(true) * 1000000) + random_int(1, 1000);
    }
}

