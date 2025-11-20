<?php

declare(strict_types=1);

namespace Voc\Admin\Interfaces\Http;

use PDO;
use Psr\Http\Message\ResponseInterface;
use Psr\Http\Message\ServerRequestInterface;
use Slim\App;
use Voc\Admin\Infrastructure\Response\JsonResponder;
use Voc\Admin\Infrastructure\Security\PasswordService;
use Voc\Admin\Infrastructure\Security\RsaDecryptor;
use Voc\Admin\Infrastructure\Security\TokenService;

/**
 * 系统用户管理相关路由：/system/user 系列接口。
 */
final class SystemUserRoutes
{
    private PDO $db;
    private TokenService $tokens;
    private RsaDecryptor $rsa;
    private PasswordService $passwords;
    private JsonResponder $responder;

    public function __construct(
        PDO $db,
        TokenService $tokens,
        RsaDecryptor $rsa,
        PasswordService $passwords,
        JsonResponder $responder
    ) {
        $this->db = $db;
        $this->tokens = $tokens;
        $this->rsa = $rsa;
        $this->passwords = $passwords;
        $this->responder = $responder;
    }

    public function register(App $app): void
    {
        $app->get('/system/user', fn (ServerRequestInterface $r) => $this->listUserPage($r));
        $app->get('/system/user/list', fn (ServerRequestInterface $r) => $this->listAllUser($r));
        $app->get('/system/user/{id}', fn (ServerRequestInterface $r, array $args) => $this->getUserDetail($r, $args));
        $app->post('/system/user', fn (ServerRequestInterface $r) => $this->createUser($r));
        $app->put('/system/user/{id}', fn (ServerRequestInterface $r, array $a) => $this->updateUser($r, $a));
        $app->delete('/system/user', fn (ServerRequestInterface $r) => $this->deleteUser($r));
        $app->patch('/system/user/{id}/password', fn (ServerRequestInterface $r, array $a) => $this->resetPassword($r, $a));
        $app->patch('/system/user/{id}/role', fn (ServerRequestInterface $r, array $a) => $this->updateUserRole($r, $a));
        $app->get('/system/user/export', fn (ServerRequestInterface $r) => $this->exportUser($r));
        $app->get('/system/user/import/template', fn (ServerRequestInterface $r) => $this->downloadImportTemplate($r));
        $app->post('/system/user/import/parse', fn (ServerRequestInterface $r) => $this->parseImportUser($r));
        $app->post('/system/user/import', fn (ServerRequestInterface $r) => $this->importUser($r));
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

    private function listUserPage(ServerRequestInterface $request): ResponseInterface
    {
        $params = $request->getQueryParams();
        $page = max(1, (int)($params['page'] ?? 1));
        $size = max(1, (int)($params['size'] ?? 10));
        $desc = trim((string)($params['description'] ?? ''));
        $statusStr = trim((string)($params['status'] ?? ''));
        $deptStr = trim((string)($params['deptId'] ?? ''));

        $where = 'WHERE 1=1';
        $args = [];
        if ($desc !== '') {
            $where .= ' AND (u.username ILIKE :desc OR u.nickname ILIKE :desc OR COALESCE(u.description,\'\') ILIKE :desc)';
            $args[':desc'] = '%' . $desc . '%';
        }
        if ($statusStr !== '') {
            $where .= ' AND u.status = :status';
            $args[':status'] = (int)$statusStr;
        }
        if ($deptStr !== '') {
            $where .= ' AND u.dept_id = :deptId';
            $args[':deptId'] = (int)$deptStr;
        }

        $countSql = 'SELECT COUNT(*) FROM sys_user AS u ' . $where;
        $stmt = $this->db->prepare($countSql);
        $stmt->execute($args);
        $total = (int)$stmt->fetchColumn();
        if ($total === 0) {
            return $this->responder->ok(['list' => [], 'total' => 0]);
        }

        $offset = ($page - 1) * $size;
        $query = "
SELECT u.id,
       u.username,
       u.nickname,
       COALESCE(u.avatar, ''),
       u.gender,
       COALESCE(u.email, ''),
       COALESCE(u.phone, ''),
       COALESCE(u.description, ''),
       u.status,
       u.is_system,
       u.dept_id,
       COALESCE(d.name, ''),
       u.create_time,
       COALESCE(cu.nickname, ''),
       u.update_time,
       COALESCE(uu.nickname, '')
FROM sys_user AS u
LEFT JOIN sys_dept AS d ON d.id = u.dept_id
LEFT JOIN sys_user AS cu ON cu.id = u.create_user
LEFT JOIN sys_user AS uu ON uu.id = u.update_user
{$where}
ORDER BY u.id DESC
LIMIT :limit OFFSET :offset";

        $stmt = $this->db->prepare($query);
        foreach ($args as $k => $v) {
            $stmt->bindValue($k, $v);
        }
        $stmt->bindValue(':limit', $size, PDO::PARAM_INT);
        $stmt->bindValue(':offset', $offset, PDO::PARAM_INT);
        $stmt->execute();
        $rows = $stmt->fetchAll();

        $users = [];
        foreach ($rows as $row) {
            $item = [
                'id'               => (int)$row['id'],
                'username'         => (string)$row['username'],
                'nickname'         => (string)$row['nickname'],
                'avatar'           => (string)$row['avatar'],
                'gender'           => (int)$row['gender'],
                'email'            => (string)$row['email'],
                'phone'            => (string)$row['phone'],
                'description'      => (string)$row['description'],
                'status'           => (int)$row['status'],
                'isSystem'         => (bool)$row['is_system'],
                'deptId'           => (int)$row['dept_id'],
                'deptName'         => (string)$row['name'],
                'createUserString' => (string)$row['nickname'],
                'createTime'       => (string)$row['create_time'],
                'updateUserString' => (string)$row['nickname'],
                'updateTime'       => (string)$row['update_time'],
                'disabled'         => (bool)$row['is_system'],
                'roleIds'          => [],
                'roleNames'        => [],
            ];
            $users[] = $item;
        }

        $this->fillUserRoles($users);

        return $this->responder->ok([
            'list'  => $users,
            'total' => $total,
        ]);
    }

    private function listAllUser(ServerRequestInterface $request): ResponseInterface
    {
        $params = $request->getQueryParams();
        $idStrs = (array)($params['userIds'] ?? []);
        $ids = [];
        foreach ($idStrs as $s) {
            $v = (int)$s;
            if ($v > 0) {
                $ids[] = $v;
            }
        }

        if ($ids) {
            $in = implode(',', array_fill(0, count($ids), '?'));
            $sql = "
SELECT u.id,
       u.username,
       u.nickname,
       COALESCE(u.avatar, ''),
       u.gender,
       COALESCE(u.email, ''),
       COALESCE(u.phone, ''),
       COALESCE(u.description, ''),
       u.status,
       u.is_system,
       u.dept_id,
       COALESCE(d.name, ''),
       u.create_time,
       COALESCE(cu.nickname, ''),
       u.update_time,
       COALESCE(uu.nickname, '')
FROM sys_user AS u
LEFT JOIN sys_dept AS d ON d.id = u.dept_id
LEFT JOIN sys_user AS cu ON cu.id = u.create_user
LEFT JOIN sys_user AS uu ON uu.id = u.update_user
WHERE u.id IN ($in)
ORDER BY u.id DESC";
            $stmt = $this->db->prepare($sql);
            $stmt->execute($ids);
        } else {
            $sql = "
SELECT u.id,
       u.username,
       u.nickname,
       COALESCE(u.avatar, ''),
       u.gender,
       COALESCE(u.email, ''),
       COALESCE(u.phone, ''),
       COALESCE(u.description, ''),
       u.status,
       u.is_system,
       u.dept_id,
       COALESCE(d.name, ''),
       u.create_time,
       COALESCE(cu.nickname, ''),
       u.update_time,
       COALESCE(uu.nickname, '')
FROM sys_user AS u
LEFT JOIN sys_dept AS d ON d.id = u.dept_id
LEFT JOIN sys_user AS cu ON cu.id = u.create_user
LEFT JOIN sys_user AS uu ON uu.id = u.update_user
ORDER BY u.id DESC";
            $stmt = $this->db->query($sql);
        }

        $rows = $stmt->fetchAll();
        $users = [];
        foreach ($rows as $row) {
            $item = [
                'id'               => (int)$row['id'],
                'username'         => (string)$row['username'],
                'nickname'         => (string)$row['nickname'],
                'avatar'           => (string)$row['avatar'],
                'gender'           => (int)$row['gender'],
                'email'            => (string)$row['email'],
                'phone'            => (string)$row['phone'],
                'description'      => (string)$row['description'],
                'status'           => (int)$row['status'],
                'isSystem'         => (bool)$row['is_system'],
                'deptId'           => (int)$row['dept_id'],
                'deptName'         => (string)$row['name'],
                'createUserString' => (string)$row['nickname'],
                'createTime'       => (string)$row['create_time'],
                'updateUserString' => (string)$row['nickname'],
                'updateTime'       => (string)$row['update_time'],
                'disabled'         => (bool)$row['is_system'],
                'roleIds'          => [],
                'roleNames'        => [],
            ];
            $users[] = $item;
        }

        $this->fillUserRoles($users);

        return $this->responder->ok($users);
    }

    /**
     * @param array<int,array<string,mixed>> $users
     */
    private function fillUserRoles(array &$users): void
    {
        if (!$users) {
            return;
        }
        $userIds = [];
        $mapIdx = [];
        foreach ($users as $i => $u) {
            $id = (int)$u['id'];
            $userIds[$id] = $id;
            $mapIdx[$id] = $i;
        }
        $ids = array_values($userIds);
        if (!$ids) {
            return;
        }

        $in = implode(',', array_fill(0, count($ids), '?'));
        $sql = "
SELECT ur.user_id, ur.role_id, r.name
FROM sys_user_role AS ur
JOIN sys_role AS r ON r.id = ur.role_id
WHERE ur.user_id IN ($in)";
        $stmt = $this->db->prepare($sql);
        $stmt->execute($ids);
        $rows = $stmt->fetchAll();

        $roleMap = [];
        foreach ($rows as $row) {
            $uid = (int)$row['user_id'];
            $rid = (int)$row['role_id'];
            $name = (string)$row['name'];
            if (!isset($roleMap[$uid])) {
                $roleMap[$uid] = ['ids' => [], 'names' => []];
            }
            $roleMap[$uid]['ids'][] = $rid;
            $roleMap[$uid]['names'][] = $name;
        }

        foreach ($roleMap as $uid => $info) {
            if (!isset($mapIdx[$uid])) {
                continue;
            }
            $i = $mapIdx[$uid];
            $users[$i]['roleIds'] = $info['ids'];
            $users[$i]['roleNames'] = $info['names'];
        }
    }

    private function getUserDetail(ServerRequestInterface $request, array $args): ResponseInterface
    {
        $id = (int)($args['id'] ?? 0);
        if ($id <= 0) {
            return $this->responder->fail('400', 'ID 参数不正确');
        }

        $sql = "
SELECT u.id,
       u.username,
       u.nickname,
       COALESCE(u.avatar, ''),
       u.gender,
       COALESCE(u.email, ''),
       COALESCE(u.phone, ''),
       COALESCE(u.description, ''),
       u.status,
       u.is_system,
       u.dept_id,
       COALESCE(d.name, ''),
       u.pwd_reset_time,
       u.create_time,
       COALESCE(cu.nickname, ''),
       u.update_time,
       COALESCE(uu.nickname, '')
FROM sys_user AS u
LEFT JOIN sys_dept AS d ON d.id = u.dept_id
LEFT JOIN sys_user AS cu ON cu.id = u.create_user
LEFT JOIN sys_user AS uu ON uu.id = u.update_user
WHERE u.id = :id";
        $stmt = $this->db->prepare($sql);
        $stmt->execute([':id' => $id]);
        $row = $stmt->fetch();
        if ($row === false) {
            return $this->responder->fail('404', '用户不存在');
        }

        $pwdResetTime = $row['pwd_reset_time'] ? (string)$row['pwd_reset_time'] : '';
        $base = [
            'id'               => (int)$row['id'],
            'username'         => (string)$row['username'],
            'nickname'         => (string)$row['nickname'],
            'avatar'           => (string)$row['avatar'],
            'gender'           => (int)$row['gender'],
            'email'            => (string)$row['email'],
            'phone'            => (string)$row['phone'],
            'description'      => (string)$row['description'],
            'status'           => (int)$row['status'],
            'isSystem'         => (bool)$row['is_system'],
            'deptId'           => (int)$row['dept_id'],
            'deptName'         => (string)$row['name'],
            'createUserString' => (string)$row['nickname'],
            'createTime'       => (string)$row['create_time'],
            'updateUserString' => (string)$row['nickname'],
            'updateTime'       => (string)$row['update_time'],
            'disabled'         => (bool)$row['is_system'],
            'roleIds'          => [],
            'roleNames'        => [],
        ];
        $users = [$base];
        $this->fillUserRoles($users);
        $user = $users[0];

        $resp = $user;
        $resp['pwdResetTime'] = $pwdResetTime;

        return $this->responder->ok($resp);
    }

    private function createUser(ServerRequestInterface $request): ResponseInterface
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

        $username = trim((string)($data['username'] ?? ''));
        $nickname = trim((string)($data['nickname'] ?? ''));
        $passwordEnc = trim((string)($data['password'] ?? ''));
        $deptId = (int)($data['deptId'] ?? 0);
        $gender = (int)($data['gender'] ?? 0);
        $email = (string)($data['email'] ?? '');
        $phone = (string)($data['phone'] ?? '');
        $avatar = (string)($data['avatar'] ?? '');
        $description = (string)($data['description'] ?? '');
        $status = (int)($data['status'] ?? 1);
        $roleIds = (array)($data['roleIds'] ?? []);

        if ($username === '' || $nickname === '') {
            return $this->responder->fail('400', '用户名和昵称不能为空');
        }
        if ($deptId === 0) {
            return $this->responder->fail('400', '所属部门不能为空');
        }
        if ($passwordEnc === '') {
            return $this->responder->fail('400', '密码不能为空');
        }

        try {
            $rawPwd = $this->rsa->decryptBase64($passwordEnc);
        } catch (\Throwable $e) {
            return $this->responder->fail('400', '密码解密失败');
        }
        if (strlen($rawPwd) < 8 || strlen($rawPwd) > 32) {
            return $this->responder->fail('400', '密码长度为 8-32 个字符，至少包含字母和数字');
        }
        $hasLetter = preg_match('/[A-Za-z]/', $rawPwd) === 1;
        $hasDigit = preg_match('/[0-9]/', $rawPwd) === 1;
        if (!$hasLetter || !$hasDigit) {
            return $this->responder->fail('400', '密码长度为 8-32 个字符，至少包含字母和数字');
        }

        try {
            $encodedPwd = $this->passwords->hash($rawPwd);
        } catch (\Throwable $e) {
            return $this->responder->fail('500', '密码加密失败');
        }

        $now = (new \DateTimeImmutable())->format('Y-m-d H:i:s');
        $newId = $this->nextId();

        try {
            $this->db->beginTransaction();
            $sql = "
INSERT INTO sys_user (
    id, username, nickname, password, gender, email, phone, avatar,
    description, status, is_system, pwd_reset_time, dept_id,
    create_user, create_time
) VALUES (
    :id, :username, :nickname, :password, :gender, :email, :phone, :avatar,
    :description, :status, FALSE, :pwd_reset_time, :dept_id,
    :create_user, :create_time
)";
            $stmt = $this->db->prepare($sql);
            $stmt->execute([
                ':id'            => $newId,
                ':username'      => $username,
                ':nickname'      => $nickname,
                ':password'      => $encodedPwd,
                ':gender'        => $gender,
                ':email'         => $email,
                ':phone'         => $phone,
                ':avatar'        => $avatar,
                ':description'   => $description,
                ':status'        => $status ?: 1,
                ':pwd_reset_time'=> $now,
                ':dept_id'       => $deptId,
                ':create_user'   => $userId,
                ':create_time'   => $now,
            ]);

            if ($roleIds) {
                $sqlRole = "
INSERT INTO sys_user_role (id, user_id, role_id)
VALUES (:id, :user_id, :role_id)
ON CONFLICT (user_id, role_id) DO NOTHING";
                $stmtRole = $this->db->prepare($sqlRole);
                foreach ($roleIds as $rid) {
                    $rid = (int)$rid;
                    if ($rid <= 0) {
                        continue;
                    }
                    $stmtRole->execute([
                        ':id'      => $this->nextId(),
                        ':user_id' => $newId,
                        ':role_id' => $rid,
                    ]);
                }
            }

            $this->db->commit();
        } catch (\Throwable $e) {
            $this->db->rollBack();
            return $this->responder->fail('500', '新增用户失败');
        }

        return $this->responder->ok(['id' => $newId]);
    }

    private function updateUser(ServerRequestInterface $request, array $args): ResponseInterface
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

        $username = trim((string)($data['username'] ?? ''));
        $nickname = trim((string)($data['nickname'] ?? ''));
        $deptId = (int)($data['deptId'] ?? 0);
        $gender = (int)($data['gender'] ?? 0);
        $email = (string)($data['email'] ?? '');
        $phone = (string)($data['phone'] ?? '');
        $avatar = (string)($data['avatar'] ?? '');
        $description = (string)($data['description'] ?? '');
        $status = (int)($data['status'] ?? 1);
        $roleIds = (array)($data['roleIds'] ?? []);

        if ($username === '' || $nickname === '') {
            return $this->responder->fail('400', '用户名和昵称不能为空');
        }
        if ($deptId === 0) {
            return $this->responder->fail('400', '所属部门不能为空');
        }

        $now = (new \DateTimeImmutable())->format('Y-m-d H:i:s');

        try {
            $this->db->beginTransaction();
            $sql = "
UPDATE sys_user
   SET username    = :username,
       nickname    = :nickname,
       gender      = :gender,
       email       = :email,
       phone       = :phone,
       avatar      = :avatar,
       description = :description,
       status      = :status,
       dept_id     = :dept_id,
       update_user = :update_user,
       update_time = :update_time
 WHERE id          = :id";
            $stmt = $this->db->prepare($sql);
            $stmt->execute([
                ':username'    => $username,
                ':nickname'    => $nickname,
                ':gender'      => $gender,
                ':email'       => $email,
                ':phone'       => $phone,
                ':avatar'      => $avatar,
                ':description' => $description,
                ':status'      => $status ?: 1,
                ':dept_id'     => $deptId,
                ':update_user' => $userId,
                ':update_time' => $now,
                ':id'          => $id,
            ]);

            $this->db->prepare('DELETE FROM sys_user_role WHERE user_id = :id')
                ->execute([':id' => $id]);

            if ($roleIds) {
                $sqlRole = "
INSERT INTO sys_user_role (id, user_id, role_id)
VALUES (:id, :user_id, :role_id)
ON CONFLICT (user_id, role_id) DO NOTHING";
                $stmtRole = $this->db->prepare($sqlRole);
                foreach ($roleIds as $rid) {
                    $rid = (int)$rid;
                    if ($rid <= 0) {
                        continue;
                    }
                    $stmtRole->execute([
                        ':id'      => $this->nextId(),
                        ':user_id' => $id,
                        ':role_id' => $rid,
                    ]);
                }
            }

            $this->db->commit();
        } catch (\Throwable $e) {
            $this->db->rollBack();
            return $this->responder->fail('500', '修改用户失败');
        }

        return $this->responder->ok(true);
    }

    private function deleteUser(ServerRequestInterface $request): ResponseInterface
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

            // 不允许删除系统内置用户
            $sqlSys = "SELECT id FROM sys_user WHERE id IN ($in) AND is_system = TRUE";
            $stmt = $this->db->prepare($sqlSys);
            $stmt->execute($ids);
            $sysRows = $stmt->fetchAll();
            $sysIds = array_map(fn ($r) => (int)$r['id'], $sysRows);
            $deleteIds = array_diff($ids, $sysIds);
            if ($deleteIds) {
                $inDel = implode(',', array_fill(0, count($deleteIds), '?'));
                $stmt = $this->db->prepare("DELETE FROM sys_user_role WHERE user_id IN ($inDel)");
                $stmt->execute(array_values($deleteIds));
                $stmt = $this->db->prepare("DELETE FROM sys_user WHERE id IN ($inDel)");
                $stmt->execute(array_values($deleteIds));
            }

            $this->db->commit();
        } catch (\Throwable $e) {
            $this->db->rollBack();
            return $this->responder->fail('500', '删除用户失败');
        }

        return $this->responder->ok(true);
    }

    private function resetPassword(ServerRequestInterface $request, array $args): ResponseInterface
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
        $newPasswordEnc = trim((string)($data['newPassword'] ?? ''));
        if ($newPasswordEnc === '') {
            return $this->responder->fail('400', '密码不能为空');
        }

        try {
            $rawPwd = $this->rsa->decryptBase64($newPasswordEnc);
        } catch (\Throwable $e) {
            return $this->responder->fail('400', '密码解密失败');
        }
        if (strlen($rawPwd) < 8 || strlen($rawPwd) > 32) {
            return $this->responder->fail('400', '密码长度为 8-32 个字符，至少包含字母和数字');
        }
        $hasLetter = preg_match('/[A-Za-z]/', $rawPwd) === 1;
        $hasDigit = preg_match('/[0-9]/', $rawPwd) === 1;
        if (!$hasLetter || !$hasDigit) {
            return $this->responder->fail('400', '密码长度为 8-32 个字符，至少包含字母和数字');
        }

        try {
            $encodedPwd = $this->passwords->hash($rawPwd);
        } catch (\Throwable $e) {
            return $this->responder->fail('500', '密码加密失败');
        }

        $now = (new \DateTimeImmutable())->format('Y-m-d H:i:s');

        $sql = "
UPDATE sys_user
   SET password       = :password,
       pwd_reset_time = :pwd_reset_time,
       update_user    = :update_user,
       update_time    = :update_time
 WHERE id             = :id";
        $stmt = $this->db->prepare($sql);
        $stmt->execute([
            ':password'       => $encodedPwd,
            ':pwd_reset_time' => $now,
            ':update_user'    => $userId,
            ':update_time'    => $now,
            ':id'             => $id,
        ]);

        return $this->responder->ok(true);
    }

    private function updateUserRole(ServerRequestInterface $request, array $args): ResponseInterface
    {
        try {
            $this->currentUserId($request);
        } catch (\RuntimeException $e) {
            return $this->responder->fail('401', $e->getMessage());
        }

        $id = (int)($args['id'] ?? 0);
        if ($id <= 0) {
            return $this->responder->fail('400', 'ID 参数不正确');
        }

        $data = json_decode((string)$request->getBody(), true);
        $roleIds = is_array($data['roleIds'] ?? null) ? $data['roleIds'] : [];
        $roleIds = array_map('intval', $roleIds);
        $roleIds = array_values(array_filter($roleIds, fn ($v) => $v > 0));

        try {
            $this->db->beginTransaction();
            $this->db->prepare('DELETE FROM sys_user_role WHERE user_id = :id')
                ->execute([':id' => $id]);

            if ($roleIds) {
                $sqlRole = "
INSERT INTO sys_user_role (id, user_id, role_id)
VALUES (:id, :user_id, :role_id)
ON CONFLICT (user_id, role_id) DO NOTHING";
                $stmtRole = $this->db->prepare($sqlRole);
                foreach ($roleIds as $rid) {
                    $stmtRole->execute([
                        ':id'      => $this->nextId(),
                        ':user_id' => $id,
                        ':role_id' => $rid,
                    ]);
                }
            }

            $this->db->commit();
        } catch (\Throwable $e) {
            $this->db->rollBack();
            return $this->responder->fail('500', '分配角色失败');
        }

        return $this->responder->ok(true);
    }

    private function exportUser(ServerRequestInterface $request): ResponseInterface
    {
        $sql = "SELECT username, nickname, gender, COALESCE(email,''), COALESCE(phone,'') FROM sys_user ORDER BY id";
        $stmt = $this->db->query($sql);
        $rows = $stmt->fetchAll();

        $content = "username,nickname,gender,email,phone\n";
        foreach ($rows as $row) {
            $content .= sprintf(
                "%s,%s,%d,%s,%s\n",
                $row['username'],
                $row['nickname'],
                $row['gender'],
                $row['email'],
                $row['phone']
            );
        }

        $response = $this->responder->ok(null);
        $body = $response->getBody();
        $body->rewind();
        $body->write($content);
        return $response
            ->withHeader('Content-Type', 'text/csv; charset=utf-8')
            ->withHeader('Content-Disposition', 'attachment; filename="users.csv"');
    }

    private function downloadImportTemplate(ServerRequestInterface $request): ResponseInterface
    {
        $content = "username,nickname,gender,email,phone\n";
        $response = $this->responder->ok(null);
        $body = $response->getBody();
        $body->rewind();
        $body->write($content);
        return $response
            ->withHeader('Content-Type', 'text/csv; charset=utf-8')
            ->withHeader('Content-Disposition', 'attachment; filename="user_import_template.csv"');
    }

    private function parseImportUser(ServerRequestInterface $request): ResponseInterface
    {
        $resp = [
            'importKey'          => (string)(microtime(true) * 1000000),
            'totalRows'          => 0,
            'validRows'          => 0,
            'duplicateUserRows'  => 0,
            'duplicateEmailRows' => 0,
            'duplicatePhoneRows' => 0,
        ];
        return $this->responder->ok($resp);
    }

    private function importUser(ServerRequestInterface $request): ResponseInterface
    {
        $resp = [
            'totalRows'  => 0,
            'insertRows' => 0,
            'updateRows' => 0,
        ];
        return $this->responder->ok($resp);
    }

    /**
     * 简单 ID 生成器，使用时间戳 + 随机数，避免与已有数据冲突概率较低。
     */
    private function nextId(): int
    {
        return (int)(microtime(true) * 1000000) + random_int(1, 1000);
    }
}

