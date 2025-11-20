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
 * 菜单管理路由：/system/menu 系列接口。
 */
final class MenuRoutes
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
        $app->get('/system/menu/tree', fn (ServerRequestInterface $r) => $this->listMenuTree($r));
        $app->get('/system/menu/{id}', fn (ServerRequestInterface $r, array $a) => $this->getMenu($r, $a));
        $app->post('/system/menu', fn (ServerRequestInterface $r) => $this->createMenu($r));
        $app->put('/system/menu/{id}', fn (ServerRequestInterface $r, array $a) => $this->updateMenu($r, $a));
        $app->delete('/system/menu', fn (ServerRequestInterface $r) => $this->deleteMenu($r));
        $app->delete('/system/menu/cache', fn (ServerRequestInterface $r) => $this->clearMenuCache($r));
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

    private function listMenuTree(ServerRequestInterface $request): ResponseInterface
    {
        $sql = "
SELECT m.id,
       m.title,
       m.parent_id,
       m.type,
       COALESCE(m.path, ''),
       COALESCE(m.name, ''),
       COALESCE(m.component, ''),
       COALESCE(m.redirect, ''),
       COALESCE(m.icon, ''),
       COALESCE(m.is_external, FALSE),
       COALESCE(m.is_cache, FALSE),
       COALESCE(m.is_hidden, FALSE),
       COALESCE(m.permission, ''),
       COALESCE(m.sort, 0),
       COALESCE(m.status, 1),
       m.create_time,
       COALESCE(cu.nickname, ''),
       m.update_time,
       COALESCE(uu.nickname, '')
FROM sys_menu AS m
LEFT JOIN sys_user AS cu ON cu.id = m.create_user
LEFT JOIN sys_user AS uu ON uu.id = m.update_user
ORDER BY m.sort ASC, m.id ASC";
        $stmt = $this->db->query($sql);
        $rows = $stmt->fetchAll();

        $flat = [];
        foreach ($rows as $row) {
            $flat[] = [
                'id'               => (int)$row['id'],
                'title'            => (string)$row['title'],
                'parentId'         => (int)$row['parent_id'],
                'type'             => (int)$row['type'],
                'path'             => (string)$row['path'],
                'name'             => (string)$row['name'],
                'component'        => (string)$row['component'],
                'redirect'         => (string)$row['redirect'],
                'icon'             => (string)$row['icon'],
                'isExternal'       => (bool)$row['is_external'],
                'isCache'          => (bool)$row['is_cache'],
                'isHidden'         => (bool)$row['is_hidden'],
                'permission'       => (string)$row['permission'],
                'sort'             => (int)$row['sort'],
                'status'           => (int)$row['status'],
                'createUserString' => (string)$row['nickname'],
                'createTime'       => (string)$row['create_time'],
                'updateUserString' => (string)$row['nickname'],
                'updateTime'       => (string)$row['update_time'],
                'children'         => [],
            ];
        }

        if (!$flat) {
            return $this->responder->ok([]);
        }

        $nodeMap = [];
        foreach ($flat as &$item) {
            $nodeMap[$item['id']] = &$item;
        }
        unset($item);

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

    private function getMenu(ServerRequestInterface $request, array $args): ResponseInterface
    {
        $id = (int)($args['id'] ?? 0);
        if ($id <= 0) {
            return $this->responder->fail('400', 'ID 参数不正确');
        }

        $sql = "
SELECT m.id,
       m.title,
       m.parent_id,
       m.type,
       COALESCE(m.path, ''),
       COALESCE(m.name, ''),
       COALESCE(m.component, ''),
       COALESCE(m.redirect, ''),
       COALESCE(m.icon, ''),
       COALESCE(m.is_external, FALSE),
       COALESCE(m.is_cache, FALSE),
       COALESCE(m.is_hidden, FALSE),
       COALESCE(m.permission, ''),
       COALESCE(m.sort, 0),
       COALESCE(m.status, 1),
       m.create_time,
       COALESCE(cu.nickname, ''),
       m.update_time,
       COALESCE(uu.nickname, '')
FROM sys_menu AS m
LEFT JOIN sys_user AS cu ON cu.id = m.create_user
LEFT JOIN sys_user AS uu ON uu.id = m.update_user
WHERE m.id = :id";
        $stmt = $this->db->prepare($sql);
        $stmt->execute([':id' => $id]);
        $row = $stmt->fetch();
        if ($row === false) {
            return $this->responder->fail('404', '菜单不存在');
        }

        $item = [
            'id'               => (int)$row['id'],
            'title'            => (string)$row['title'],
            'parentId'         => (int)$row['parent_id'],
            'type'             => (int)$row['type'],
            'path'             => (string)$row['path'],
            'name'             => (string)$row['name'],
            'component'        => (string)$row['component'],
            'redirect'         => (string)$row['redirect'],
            'icon'             => (string)$row['icon'],
            'isExternal'       => (bool)$row['is_external'],
            'isCache'          => (bool)$row['is_cache'],
            'isHidden'         => (bool)$row['is_hidden'],
            'permission'       => (string)$row['permission'],
            'sort'             => (int)$row['sort'],
            'status'           => (int)$row['status'],
            'createUserString' => (string)$row['nickname'],
            'createTime'       => (string)$row['create_time'],
            'updateUserString' => (string)$row['nickname'],
            'updateTime'       => (string)$row['update_time'],
            'children'         => [],
        ];

        return $this->responder->ok($item);
    }

    private function createMenu(ServerRequestInterface $request): ResponseInterface
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

        $type = (int)($data['type'] ?? 1);
        $title = trim((string)($data['title'] ?? ''));
        $icon = (string)($data['icon'] ?? '');
        $sort = (int)($data['sort'] ?? 999);
        $permission = (string)($data['permission'] ?? '');
        $path = trim((string)($data['path'] ?? ''));
        $name = trim((string)($data['name'] ?? ''));
        $component = trim((string)($data['component'] ?? ''));
        $redirect = (string)($data['redirect'] ?? '');
        $parentId = (int)($data['parentId'] ?? 0);
        $status = (int)($data['status'] ?? 1);
        $isExternal = (bool)($data['isExternal'] ?? false);
        $isCache = (bool)($data['isCache'] ?? false);
        $isHidden = (bool)($data['isHidden'] ?? false);

        if ($title === '') {
            return $this->responder->fail('400', '菜单标题不能为空');
        }

        if ($isExternal) {
            if (!(str_starts_with($path, 'http://') || str_starts_with($path, 'https://'))) {
                return $this->responder->fail('400', '路由地址格式不正确，请以 http:// 或 https:// 开头');
            }
        } else {
            if (str_starts_with($path, 'http://') || str_starts_with($path, 'https://')) {
                return $this->responder->fail('400', '路由地址格式不正确');
            }
            if ($path !== '' && !str_starts_with($path, '/')) {
                $path = '/' . $path;
            }
            $name = ltrim($name, '/');
            $component = ltrim($component, '/');
        }

        if ($sort <= 0) {
            $sort = 999;
        }
        if ($status === 0) {
            $status = 1;
        }

        $now = (new \DateTimeImmutable())->format('Y-m-d H:i:s');
        $newId = $this->nextId();

        $sql = "
INSERT INTO sys_menu (
    id, title, parent_id, type, path, name, component, redirect,
    icon, is_external, is_cache, is_hidden, permission, sort, status,
    create_user, create_time
) VALUES (
    :id, :title, :parent_id, :type, :path, :name, :component, :redirect,
    :icon, :is_external, :is_cache, :is_hidden, :permission, :sort, :status,
    :create_user, :create_time
)";
        $stmt = $this->db->prepare($sql);
        $stmt->execute([
            ':id'          => $newId,
            ':title'       => $title,
            ':parent_id'   => $parentId,
            ':type'        => $type,
            ':path'        => $path,
            ':name'        => $name,
            ':component'   => $component,
            ':redirect'    => $redirect,
            ':icon'        => $icon,
            ':is_external' => $isExternal,
            ':is_cache'    => $isCache,
            ':is_hidden'   => $isHidden,
            ':permission'  => $permission,
            ':sort'        => $sort,
            ':status'      => $status,
            ':create_user' => $userId,
            ':create_time' => $now,
        ]);

        return $this->responder->ok(['id' => $newId]);
    }

    private function updateMenu(ServerRequestInterface $request, array $args): ResponseInterface
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

        $title = trim((string)($data['title'] ?? ''));
        $icon = (string)($data['icon'] ?? '');
        $sort = (int)($data['sort'] ?? 999);
        $permission = (string)($data['permission'] ?? '');
        $path = trim((string)($data['path'] ?? ''));
        $name = trim((string)($data['name'] ?? ''));
        $component = trim((string)($data['component'] ?? ''));
        $redirect = (string)($data['redirect'] ?? '');
        $parentId = (int)($data['parentId'] ?? 0);
        $status = (int)($data['status'] ?? 1);
        $isExternal = (bool)($data['isExternal'] ?? false);
        $isCache = (bool)($data['isCache'] ?? false);
        $isHidden = (bool)($data['isHidden'] ?? false);
        $type = (int)($data['type'] ?? 1);

        if ($title === '') {
            return $this->responder->fail('400', '菜单标题不能为空');
        }

        if ($isExternal) {
            if (!(str_starts_with($path, 'http://') || str_starts_with($path, 'https://'))) {
                return $this->responder->fail('400', '路由地址格式不正确，请以 http:// 或 https:// 开头');
            }
        } else {
            if (str_starts_with($path, 'http://') || str_starts_with($path, 'https://')) {
                return $this->responder->fail('400', '路由地址格式不正确');
            }
            if ($path !== '' && !str_starts_with($path, '/')) {
                $path = '/' . $path;
            }
            $name = ltrim($name, '/');
            $component = ltrim($component, '/');
        }

        if ($sort <= 0) {
            $sort = 999;
        }
        if ($status === 0) {
            $status = 1;
        }

        $now = (new \DateTimeImmutable())->format('Y-m-d H:i:s');

        $sql = "
UPDATE sys_menu
   SET title       = :title,
       parent_id   = :parent_id,
       type        = :type,
       path        = :path,
       name        = :name,
       component   = :component,
       redirect    = :redirect,
       icon        = :icon,
       is_external = :is_external,
       is_cache    = :is_cache,
       is_hidden   = :is_hidden,
       permission  = :permission,
       sort        = :sort,
       status      = :status,
       update_user = :update_user,
       update_time = :update_time
 WHERE id          = :id";
        $stmt = $this->db->prepare($sql);
        $stmt->execute([
            ':title'       => $title,
            ':parent_id'   => $parentId,
            ':type'        => $type,
            ':path'        => $path,
            ':name'        => $name,
            ':component'   => $component,
            ':redirect'    => $redirect,
            ':icon'        => $icon,
            ':is_external' => $isExternal,
            ':is_cache'    => $isCache,
            ':is_hidden'   => $isHidden,
            ':permission'  => $permission,
            ':sort'        => $sort,
            ':status'      => $status,
            ':update_user' => $userId,
            ':update_time' => $now,
            ':id'          => $id,
        ]);

        return $this->responder->ok(true);
    }

    private function deleteMenu(ServerRequestInterface $request): ResponseInterface
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

        $stmt = $this->db->query('SELECT id, parent_id FROM sys_menu');
        $rows = $stmt->fetchAll();
        $childrenOf = [];
        foreach ($rows as $row) {
            $pid = (int)$row['parent_id'];
            $cid = (int)$row['id'];
            $childrenOf[$pid][] = $cid;
        }

        $seen = [];
        $collect = function (int $id) use (&$collect, &$seen, $childrenOf): void {
            if (isset($seen[$id])) {
                return;
            }
            $seen[$id] = true;
            foreach ($childrenOf[$id] ?? [] as $child) {
                $collect($child);
            }
        };
        foreach ($ids as $id) {
            $collect($id);
        }
        $allIds = array_keys($seen);
        if (!$allIds) {
            return $this->responder->ok(true);
        }

        try {
            $this->db->beginTransaction();
            $in = implode(',', array_fill(0, count($allIds), '?'));
            $stmt = $this->db->prepare("DELETE FROM sys_role_menu WHERE menu_id IN ($in)");
            $stmt->execute($allIds);
            $stmt = $this->db->prepare("DELETE FROM sys_menu WHERE id IN ($in)");
            $stmt->execute($allIds);
            $this->db->commit();
        } catch (\Throwable $e) {
            $this->db->rollBack();
            return $this->responder->fail('500', '删除菜单失败');
        }

        return $this->responder->ok(true);
    }

    private function clearMenuCache(ServerRequestInterface $request): ResponseInterface
    {
        return $this->responder->ok(true);
    }

    private function nextId(): int
    {
        return (int)(microtime(true) * 1000000) + random_int(1, 1000);
    }
}

