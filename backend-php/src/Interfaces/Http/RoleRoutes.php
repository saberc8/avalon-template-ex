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
 * 角色管理路由：/system/role 系列接口。
 */
final class RoleRoutes
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
        $app->get('/system/role/list', fn (ServerRequestInterface $r) => $this->listRole($r));
        $app->get('/system/role/{id}', fn (ServerRequestInterface $r, array $a) => $this->getRole($r, $a));
        $app->post('/system/role', fn (ServerRequestInterface $r) => $this->createRole($r));
        $app->put('/system/role/{id}', fn (ServerRequestInterface $r, array $a) => $this->updateRole($r, $a));
        $app->delete('/system/role', fn (ServerRequestInterface $r) => $this->deleteRole($r));

        $app->put('/system/role/{id}/permission', fn (ServerRequestInterface $r, array $a) => $this->updateRolePermission($r, $a));
        $app->get('/system/role/{id}/user', fn (ServerRequestInterface $r, array $a) => $this->pageRoleUser($r, $a));
        $app->post('/system/role/{id}/user', fn (ServerRequestInterface $r, array $a) => $this->assignToUsers($r, $a));
        $app->delete('/system/role/user', fn (ServerRequestInterface $r) => $this->unassignFromUsers($r));
        $app->get('/system/role/{id}/user/id', fn (ServerRequestInterface $r, array $a) => $this->listRoleUserIDs($r, $a));
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

    private function listRole(ServerRequestInterface $request): ResponseInterface
    {
        $params = $request->getQueryParams();
        $descFilter = trim((string)($params['description'] ?? ''));

        $sql = "
SELECT r.id,
       r.name,
       r.code,
       COALESCE(r.sort, 999),
       COALESCE(r.description, ''),
       COALESCE(r.data_scope, 4),
       COALESCE(r.is_system, FALSE),
       r.create_time,
       COALESCE(cu.nickname, ''),
       r.update_time,
       COALESCE(uu.nickname, '')
FROM sys_role AS r
LEFT JOIN sys_user AS cu ON cu.id = r.create_user
LEFT JOIN sys_user AS uu ON uu.id = r.update_user
ORDER BY r.sort ASC, r.id ASC";
        $stmt = $this->db->query($sql);
        $rows = $stmt->fetchAll();

        $list = [];
        foreach ($rows as $row) {
            $item = [
                'id'               => (int)$row['id'],
                'name'             => (string)$row['name'],
                'code'             => (string)$row['code'],
                'sort'             => (int)$row['sort'],
                'description'      => (string)$row['description'],
                'dataScope'        => (int)$row['data_scope'],
                'isSystem'         => (bool)$row['is_system'],
                'createUserString' => (string)$row['nickname'],
                'createTime'       => (string)$row['create_time'],
                'updateUserString' => (string)$row['nickname'],
                'updateTime'       => (string)$row['update_time'],
                'disabled'         => (bool)$row['is_system'] && (string)$row['code'] === 'admin',
            ];
            if ($descFilter !== '' &&
                !str_contains($item['name'], $descFilter) &&
                !str_contains($item['description'], $descFilter)
            ) {
                continue;
            }
            $list[] = $item;
        }

        return $this->responder->ok($list);
    }

    private function getRole(ServerRequestInterface $request, array $args): ResponseInterface
    {
        $id = (int)($args['id'] ?? 0);
        if ($id <= 0) {
            return $this->responder->fail('400', 'ID 参数不正确');
        }

        $sql = "
SELECT r.id,
       r.name,
       r.code,
       COALESCE(r.sort, 999),
       COALESCE(r.description, ''),
       COALESCE(r.data_scope, 4),
       COALESCE(r.is_system, FALSE),
       COALESCE(r.menu_check_strictly, TRUE),
       COALESCE(r.dept_check_strictly, TRUE),
       r.create_time,
       COALESCE(cu.nickname, ''),
       r.update_time,
       COALESCE(uu.nickname, '')
FROM sys_role AS r
LEFT JOIN sys_user AS cu ON cu.id = r.create_user
LEFT JOIN sys_user AS uu ON uu.id = r.update_user
WHERE r.id = :id";
        $stmt = $this->db->prepare($sql);
        $stmt->execute([':id' => $id]);
        $row = $stmt->fetch();
        if ($row === false) {
            return $this->responder->fail('404', '角色不存在');
        }

        $base = [
            'id'               => (int)$row['id'],
            'name'             => (string)$row['name'],
            'code'             => (string)$row['code'],
            'sort'             => (int)$row['sort'],
            'description'      => (string)$row['description'],
            'dataScope'        => (int)$row['data_scope'],
            'isSystem'         => (bool)$row['is_system'],
            'createUserString' => (string)$row['nickname'],
            'createTime'       => (string)$row['create_time'],
            'updateUserString' => (string)$row['nickname'],
            'updateTime'       => (string)$row['update_time'],
            'disabled'         => (bool)$row['is_system'] && (string)$row['code'] === 'admin',
        ];

        $menuRows = $this->db->prepare('SELECT menu_id FROM sys_role_menu WHERE role_id = :id');
        $menuRows->execute([':id' => $id]);
        $menuIds = array_map(fn ($r) => (int)$r['menu_id'], $menuRows->fetchAll());

        $deptRows = $this->db->prepare('SELECT dept_id FROM sys_role_dept WHERE role_id = :id');
        $deptRows->execute([':id' => $id]);
        $deptIds = array_map(fn ($r) => (int)$r['dept_id'], $deptRows->fetchAll());

        $resp = $base + [
            'menuIds'          => $menuIds,
            'deptIds'          => $deptIds,
            'menuCheckStrictly'=> (bool)$row['menu_check_strictly'],
            'deptCheckStrictly'=> (bool)$row['dept_check_strictly'],
        ];

        return $this->responder->ok($resp);
    }

    private function createRole(ServerRequestInterface $request): ResponseInterface
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

        $name = trim((string)($data['name'] ?? ''));
        $code = trim((string)($data['code'] ?? ''));
        $sort = (int)($data['sort'] ?? 999);
        $desc = (string)($data['description'] ?? '');
        $dataScope = (int)($data['dataScope'] ?? 4);
        $deptIds = (array)($data['deptIds'] ?? []);
        $deptCheckStrict = (bool)($data['deptCheckStrictly'] ?? true);

        if ($name === '' || $code === '') {
            return $this->responder->fail('400', '名称和编码不能为空');
        }
        if ($sort <= 0) {
            $sort = 999;
        }
        if ($dataScope === 0) {
            $dataScope = 4;
        }

        $now = (new \DateTimeImmutable())->format('Y-m-d H:i:s');
        $newId = $this->nextId();

        try {
            $this->db->beginTransaction();
            $sql = "
INSERT INTO sys_role (
    id, name, code, data_scope, description, sort,
    is_system, menu_check_strictly, dept_check_strictly,
    create_user, create_time
) VALUES (
    :id, :name, :code, :data_scope, :description, :sort,
    FALSE, TRUE, :dept_check,
    :create_user, :create_time
)";
            $stmt = $this->db->prepare($sql);
            $stmt->execute([
                ':id'          => $newId,
                ':name'        => $name,
                ':code'        => $code,
                ':data_scope'  => $dataScope,
                ':description' => $desc,
                ':sort'        => $sort,
                ':dept_check'  => $deptCheckStrict,
                ':create_user' => $userId,
                ':create_time' => $now,
            ]);

            if ($deptIds) {
                $sqlDept = "
INSERT INTO sys_role_dept (role_id, dept_id)
VALUES (:role_id, :dept_id)
ON CONFLICT DO NOTHING";
                $stmtDept = $this->db->prepare($sqlDept);
                foreach ($deptIds as $did) {
                    $did = (int)$did;
                    if ($did <= 0) {
                        continue;
                    }
                    $stmtDept->execute([
                        ':role_id' => $newId,
                        ':dept_id' => $did,
                    ]);
                }
            }

            $this->db->commit();
        } catch (\Throwable $e) {
            $this->db->rollBack();
            return $this->responder->fail('500', '新增角色失败');
        }

        return $this->responder->ok(['id' => $newId]);
    }

    private function updateRole(ServerRequestInterface $request, array $args): ResponseInterface
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

        $name = trim((string)($data['name'] ?? ''));
        $sort = (int)($data['sort'] ?? 999);
        $desc = (string)($data['description'] ?? '');
        $dataScope = (int)($data['dataScope'] ?? 4);
        $deptIds = (array)($data['deptIds'] ?? []);
        $deptCheckStrict = (bool)($data['deptCheckStrictly'] ?? true);

        if ($name === '') {
            return $this->responder->fail('400', '名称不能为空');
        }
        if ($sort <= 0) {
            $sort = 999;
        }
        if ($dataScope === 0) {
            $dataScope = 4;
        }

        $now = (new \DateTimeImmutable())->format('Y-m-d H:i:s');

        try {
            $this->db->beginTransaction();
            $sql = "
UPDATE sys_role
   SET name               = :name,
       description        = :description,
       sort               = :sort,
       data_scope         = :data_scope,
       dept_check_strictly= :dept_check,
       update_user        = :update_user,
       update_time        = :update_time
 WHERE id                 = :id";
            $stmt = $this->db->prepare($sql);
            $stmt->execute([
                ':name'        => $name,
                ':description' => $desc,
                ':sort'        => $sort,
                ':data_scope'  => $dataScope,
                ':dept_check'  => $deptCheckStrict,
                ':update_user' => $userId,
                ':update_time' => $now,
                ':id'          => $id,
            ]);

            $this->db->prepare('DELETE FROM sys_role_dept WHERE role_id = :id')
                ->execute([':id' => $id]);

            if ($deptIds) {
                $sqlDept = "
INSERT INTO sys_role_dept (role_id, dept_id)
VALUES (:role_id, :dept_id)
ON CONFLICT DO NOTHING";
                $stmtDept = $this->db->prepare($sqlDept);
                foreach ($deptIds as $did) {
                    $did = (int)$did;
                    if ($did <= 0) {
                        continue;
                    }
                    $stmtDept->execute([
                        ':role_id' => $id,
                        ':dept_id' => $did,
                    ]);
                }
            }

            $this->db->commit();
        } catch (\Throwable $e) {
            $this->db->rollBack();
            return $this->responder->fail('500', '修改角色失败');
        }

        return $this->responder->ok(true);
    }

    private function deleteRole(ServerRequestInterface $request): ResponseInterface
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

            // 不允许删除系统内置角色 admin
            $sqlSys = "SELECT id FROM sys_role WHERE id IN ($in) AND is_system = TRUE AND code = 'admin'";
            $stmt = $this->db->prepare($sqlSys);
            $stmt->execute($ids);
            $sysRows = $stmt->fetchAll();
            $sysIds = array_map(fn ($r) => (int)$r['id'], $sysRows);
            $deleteIds = array_diff($ids, $sysIds);
            if ($deleteIds) {
                $inDel = implode(',', array_fill(0, count($deleteIds), '?'));
                $stmt = $this->db->prepare("DELETE FROM sys_user_role WHERE role_id IN ($inDel)");
                $stmt->execute(array_values($deleteIds));
                $stmt = $this->db->prepare("DELETE FROM sys_role_menu WHERE role_id IN ($inDel)");
                $stmt->execute(array_values($deleteIds));
                $stmt = $this->db->prepare("DELETE FROM sys_role_dept WHERE role_id IN ($inDel)");
                $stmt->execute(array_values($deleteIds));
                $stmt = $this->db->prepare("DELETE FROM sys_role WHERE id IN ($inDel)");
                $stmt->execute(array_values($deleteIds));
            }

            $this->db->commit();
        } catch (\Throwable $e) {
            $this->db->rollBack();
            return $this->responder->fail('500', '删除角色失败');
        }

        return $this->responder->ok(true);
    }

    private function updateRolePermission(ServerRequestInterface $request, array $args): ResponseInterface
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
        $menuIds = is_array($data['menuIds'] ?? null) ? $data['menuIds'] : [];
        $menuIds = array_map('intval', $menuIds);
        $menuIds = array_values(array_filter($menuIds, fn ($v) => $v > 0));
        $menuCheckStrict = (bool)($data['menuCheckStrictly'] ?? true);

        $now = (new \DateTimeImmutable())->format('Y-m-d H:i:s');

        try {
            $this->db->beginTransaction();
            $this->db->prepare('DELETE FROM sys_role_menu WHERE role_id = :id')
                ->execute([':id' => $id]);

            if ($menuIds) {
                $sqlMenu = "
INSERT INTO sys_role_menu (role_id, menu_id)
VALUES (:role_id, :menu_id)
ON CONFLICT DO NOTHING";
                $stmtMenu = $this->db->prepare($sqlMenu);
                foreach ($menuIds as $mid) {
                    $stmtMenu->execute([
                        ':role_id' => $id,
                        ':menu_id' => $mid,
                    ]);
                }
            }

            $sql = "
UPDATE sys_role
   SET menu_check_strictly = :menu_check,
       update_user         = :update_user,
       update_time         = :update_time
 WHERE id                  = :id";
            $stmt = $this->db->prepare($sql);
            $stmt->execute([
                ':menu_check' => $menuCheckStrict,
                ':update_user'=> $userId,
                ':update_time'=> $now,
                ':id'         => $id,
            ]);

            $this->db->commit();
        } catch (\Throwable $e) {
            $this->db->rollBack();
            return $this->responder->fail('500', '保存角色权限失败');
        }

        return $this->responder->ok(true);
    }

    private function pageRoleUser(ServerRequestInterface $request, array $args): ResponseInterface
    {
        $roleId = (int)($args['id'] ?? 0);
        if ($roleId <= 0) {
            return $this->responder->fail('400', 'ID 参数不正确');
        }

        $params = $request->getQueryParams();
        $page = max(1, (int)($params['page'] ?? 1));
        $size = max(1, (int)($params['size'] ?? 10));
        $descFilter = trim((string)($params['description'] ?? ''));

        $sql = "
SELECT ur.id,
       ur.role_id,
       u.id,
       u.username,
       u.nickname,
       u.gender,
       u.status,
       u.is_system,
       COALESCE(u.description, ''),
       u.dept_id,
       COALESCE(d.name, '')
FROM sys_user_role AS ur
JOIN sys_user AS u ON u.id = ur.user_id
LEFT JOIN sys_dept AS d ON d.id = u.dept_id
WHERE ur.role_id = :role_id
ORDER BY ur.id DESC";
        $stmt = $this->db->prepare($sql);
        $stmt->execute([':role_id' => $roleId]);
        $rows = $stmt->fetchAll();

        $all = [];
        foreach ($rows as $row) {
            $item = [
                'id'          => (int)$row['id'],
                'roleId'      => (int)$row['role_id'],
                'userId'      => (int)$row['id'],
                'username'    => (string)$row['username'],
                'nickname'    => (string)$row['nickname'],
                'gender'      => (int)$row['gender'],
                'status'      => (int)$row['status'],
                'isSystem'    => (bool)$row['is_system'],
                'description' => (string)$row['description'],
                'deptId'      => (int)$row['dept_id'],
                'deptName'    => (string)$row['name'],
                'roleIds'     => [],
                'roleNames'   => [],
                'disabled'    => false,
            ];
            if ($descFilter !== '' &&
                !str_contains($item['username'], $descFilter) &&
                !str_contains($item['nickname'], $descFilter) &&
                !str_contains($item['description'], $descFilter)
            ) {
                continue;
            }
            $all[] = $item;
        }

        if ($all) {
            $userIds = [];
            foreach ($all as $item) {
                $userIds[$item['userId']] = $item['userId'];
            }
            $ids = array_values($userIds);
            $in = implode(',', array_fill(0, count($ids), '?'));
            $sqlRole = "
SELECT ur.user_id, ur.role_id, r.name
FROM sys_user_role AS ur
JOIN sys_role AS r ON r.id = ur.role_id
WHERE ur.user_id IN ($in)";
            $stmt = $this->db->prepare($sqlRole);
            $stmt->execute($ids);
            $rowsRole = $stmt->fetchAll();

            $roleMap = [];
            foreach ($rowsRole as $row) {
                $uid = (int)$row['user_id'];
                $rid = (int)$row['role_id'];
                $name = (string)$row['name'];
                if (!isset($roleMap[$uid])) {
                    $roleMap[$uid] = ['ids' => [], 'names' => []];
                }
                $roleMap[$uid]['ids'][] = $rid;
                $roleMap[$uid]['names'][] = $name;
            }

            foreach ($all as $i => $item) {
                $uid = $item['userId'];
                if (isset($roleMap[$uid])) {
                    $all[$i]['roleIds'] = $roleMap[$uid]['ids'];
                    $all[$i]['roleNames'] = $roleMap[$uid]['names'];
                }
                $all[$i]['disabled'] = (bool)$item['isSystem'] && (int)$item['roleId'] === 1;
            }
        }

        $total = count($all);
        $start = ($page - 1) * $size;
        if ($start > $total) {
            $start = $total;
        }
        $end = min($start + $size, $total);
        $pageList = array_slice($all, $start, $end - $start);

        return $this->responder->ok([
            'list'  => $pageList,
            'total' => $total,
        ]);
    }

    private function assignToUsers(ServerRequestInterface $request, array $args): ResponseInterface
    {
        try {
            $this->currentUserId($request);
        } catch (\RuntimeException $e) {
            return $this->responder->fail('401', $e->getMessage());
        }

        $roleId = (int)($args['id'] ?? 0);
        if ($roleId <= 0) {
            return $this->responder->fail('400', 'ID 参数不正确');
        }

        $data = json_decode((string)$request->getBody(), true);
        $userIds = is_array($data) ? $data : [];
        $userIds = array_map('intval', $userIds);
        $userIds = array_values(array_filter($userIds, fn ($v) => $v > 0));
        if (!$userIds) {
            return $this->responder->fail('400', '用户ID列表不能为空');
        }

        try {
            $this->db->beginTransaction();
            $sql = "
INSERT INTO sys_user_role (id, user_id, role_id)
VALUES (:id, :user_id, :role_id)
ON CONFLICT (user_id, role_id) DO NOTHING";
            $stmt = $this->db->prepare($sql);
            foreach ($userIds as $uid) {
                $stmt->execute([
                    ':id'      => $this->nextId(),
                    ':user_id' => $uid,
                    ':role_id' => $roleId,
                ]);
            }
            $this->db->commit();
        } catch (\Throwable $e) {
            $this->db->rollBack();
            return $this->responder->fail('500', '分配用户失败');
        }

        return $this->responder->ok(true);
    }

    private function unassignFromUsers(ServerRequestInterface $request): ResponseInterface
    {
        try {
            $this->currentUserId($request);
        } catch (\RuntimeException $e) {
            return $this->responder->fail('401', $e->getMessage());
        }

        $data = json_decode((string)$request->getBody(), true);
        $ids = is_array($data) ? $data : [];
        $ids = array_map('intval', $ids);
        $ids = array_values(array_filter($ids, fn ($v) => $v > 0));
        if (!$ids) {
            return $this->responder->fail('400', '用户角色ID列表不能为空');
        }

        $in = implode(',', array_fill(0, count($ids), '?'));
        $sql = "DELETE FROM sys_user_role WHERE id IN ($in)";
        $stmt = $this->db->prepare($sql);
        $stmt->execute($ids);

        return $this->responder->ok(true);
    }

    private function listRoleUserIDs(ServerRequestInterface $request, array $args): ResponseInterface
    {
        $roleId = (int)($args['id'] ?? 0);
        if ($roleId <= 0) {
            return $this->responder->fail('400', 'ID 参数不正确');
        }

        $stmt = $this->db->prepare('SELECT user_id FROM sys_user_role WHERE role_id = :id');
        $stmt->execute([':id' => $roleId]);
        $rows = $stmt->fetchAll();
        $ids = array_map(fn ($r) => (int)$r['user_id'], $rows);

        return $this->responder->ok($ids);
    }

    private function nextId(): int
    {
        return (int)(microtime(true) * 1000000) + random_int(1, 1000);
    }
}

