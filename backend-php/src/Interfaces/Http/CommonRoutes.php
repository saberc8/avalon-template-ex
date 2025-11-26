<?php

declare(strict_types=1);

namespace Voc\Admin\Interfaces\Http;

use PDO;
use PDOException;
use Psr\Http\Message\ResponseInterface;
use Psr\Http\Message\ServerRequestInterface;
use Slim\App;
use Voc\Admin\Infrastructure\Response\JsonResponder;

/**
 * 公共下拉/树形接口路由：/common/*。
 *
 * 目标：与 Java / Node 版本 CommonController 行为保持一致，
 * 为前端提供通用的字典、部门树、菜单树等数据。
 */
final class CommonRoutes
{
    private PDO $db;
    private JsonResponder $responder;

    public function __construct(PDO $db, JsonResponder $responder)
    {
        $this->db = $db;
        $this->responder = $responder;
    }

    /**
     * 注册路由。
     */
    public function register(App $app): void
    {
        // 站点配置字典：/common/dict/option/site
        $app->get('/common/dict/option/site', fn (ServerRequestInterface $r) => $this->listSiteOptions($r));

        // 菜单树：/common/tree/menu
        $app->get('/common/tree/menu', fn (ServerRequestInterface $r) => $this->listMenuTree($r));

        // 部门树：/common/tree/dept
        $app->get('/common/tree/dept', fn (ServerRequestInterface $r) => $this->listDeptTree($r));

        // 用户字典：/common/dict/user
        $app->get('/common/dict/user', fn (ServerRequestInterface $r) => $this->listUserDict($r));

        // 角色字典：/common/dict/role
        $app->get('/common/dict/role', fn (ServerRequestInterface $r) => $this->listRoleDict($r));

        // 通用字典：/common/dict/{code}
        $app->get('/common/dict/{code}', fn (ServerRequestInterface $r, array $a) => $this->listDictByCode($r, $a));
    }

    /**
     * 查询站点配置字典：/common/dict/option/site。
     *
     * 返回格式：[{ label: code, value }]
     */
    private function listSiteOptions(ServerRequestInterface $request): ResponseInterface
    {
        $sql = "
SELECT code,
       COALESCE(value, default_value) AS value
FROM sys_option
WHERE category = 'SITE'
ORDER BY id ASC";

        try {
            $stmt = $this->db->query($sql);
            $rows = $stmt->fetchAll();
        } catch (PDOException $e) {
            return $this->responder->fail('500', '查询站点配置失败');
        }

        $data = [];
        foreach ($rows as $row) {
            $data[] = [
                'label' => (string)$row['code'],
                'value' => (string)$row['value'],
            ];
        }

        return $this->responder->ok($data);
    }

    /**
     * 查询菜单树：/common/tree/menu。
     *
     * 返回格式：[{ key, title, disabled, children }]
     */
    private function listMenuTree(ServerRequestInterface $request): ResponseInterface
    {
        $sql = "
SELECT id,
       title,
       parent_id,
       COALESCE(sort, 999) AS sort,
       COALESCE(status, 1) AS status
FROM sys_menu
WHERE type IN (1, 2)
ORDER BY sort ASC, id ASC";

        try {
            $stmt = $this->db->query($sql);
            $rows = $stmt->fetchAll();
        } catch (PDOException $e) {
            return $this->responder->fail('500', '查询菜单树失败');
        }

        if (!$rows) {
            return $this->responder->ok([]);
        }

        $flat = [];
        foreach ($rows as $row) {
            $flat[] = [
                'id'       => (int)$row['id'],
                'title'    => (string)$row['title'],
                'parentId' => (int)$row['parent_id'],
                'sort'     => (int)$row['sort'],
                'status'   => (int)$row['status'],
            ];
        }

        $nodeMap = [];
        foreach ($flat as $m) {
            $nodeMap[$m['id']] = [
                'key'      => $m['id'],
                'title'    => $m['title'],
                'disabled' => $m['status'] !== 1,
                'children' => [],
            ];
        }

        $roots = [];
        foreach ($flat as $m) {
            $id = $m['id'];
            $pid = $m['parentId'];
            if ($pid === 0 || !isset($nodeMap[$pid])) {
                $roots[] = &$nodeMap[$id];
                continue;
            }
            $nodeMap[$pid]['children'][] = &$nodeMap[$id];
        }
        unset($m);

        return $this->responder->ok(array_values($roots));
    }

    /**
     * 查询部门树：/common/tree/dept。
     *
     * 返回格式与 /common/tree/menu 相同。
     */
    private function listDeptTree(ServerRequestInterface $request): ResponseInterface
    {
        $sql = "
SELECT id,
       name,
       parent_id,
       COALESCE(sort, 999) AS sort,
       COALESCE(status, 1) AS status,
       COALESCE(is_system, FALSE) AS is_system
FROM sys_dept
ORDER BY sort ASC, id ASC";

        try {
            $stmt = $this->db->query($sql);
            $rows = $stmt->fetchAll();
        } catch (PDOException $e) {
            return $this->responder->fail('500', '查询部门树失败');
        }

        if (!$rows) {
            return $this->responder->ok([]);
        }

        $flat = [];
        foreach ($rows as $row) {
            $flat[] = [
                'id'       => (int)$row['id'],
                'name'     => (string)$row['name'],
                'parentId' => (int)$row['parent_id'],
                'sort'     => (int)$row['sort'],
                'status'   => (int)$row['status'],
                'isSystem' => (bool)$row['is_system'],
            ];
        }

        $nodeMap = [];
        foreach ($flat as $d) {
            $nodeMap[$d['id']] = [
                'key'      => $d['id'],
                'title'    => $d['name'],
                'disabled' => false,
                'children' => [],
            ];
        }

        $roots = [];
        foreach ($flat as $d) {
            $id = $d['id'];
            $pid = $d['parentId'];
            if ($pid === 0 || !isset($nodeMap[$pid])) {
                $roots[] = &$nodeMap[$id];
                continue;
            }
            $nodeMap[$pid]['children'][] = &$nodeMap[$id];
        }
        unset($d);

        return $this->responder->ok(array_values($roots));
    }

    /**
     * 用户字典：/common/dict/user。
     *
     * - 默认只返回启用状态用户
     * - 可通过 status 参数指定状态
     * - 返回格式：[{ label: 昵称, value: 用户ID, extra: 用户名 }]
     */
    private function listUserDict(ServerRequestInterface $request): ResponseInterface
    {
        $params = $request->getQueryParams();
        $statusStr = trim((string)($params['status'] ?? ''));

        $sql = "
SELECT id,
       COALESCE(nickname, username, '') AS nickname,
       COALESCE(username, '')           AS username
FROM sys_user
WHERE status = :status
ORDER BY id ASC";
        $args = [':status' => 1];

        if ($statusStr !== '') {
            $status = (int)$statusStr;
            if ($status <= 0) {
                $status = 1;
            }
            $args[':status'] = $status;
        }

        try {
            $stmt = $this->db->prepare($sql);
            $stmt->execute($args);
            $rows = $stmt->fetchAll();
        } catch (PDOException $e) {
            return $this->responder->fail('500', '查询用户字典失败');
        }

        $data = [];
        foreach ($rows as $row) {
            $data[] = [
                'label' => (string)$row['nickname'],
                'value' => (int)$row['id'],
                'extra' => (string)$row['username'],
            ];
        }

        return $this->responder->ok($data);
    }

    /**
     * 角色字典：/common/dict/role。
     *
     * 返回格式：[{ label: 角色名称, value: 角色ID, extra: 角色编码 }]
     */
    private function listRoleDict(ServerRequestInterface $request): ResponseInterface
    {
        $sql = "
SELECT id,
       name,
       code
FROM sys_role
ORDER BY sort ASC, id ASC";

        try {
            $stmt = $this->db->query($sql);
            $rows = $stmt->fetchAll();
        } catch (PDOException $e) {
            return $this->responder->fail('500', '查询角色字典失败');
        }

        $data = [];
        foreach ($rows as $row) {
            $data[] = [
                'label' => (string)$row['name'],
                'value' => (int)$row['id'],
                'extra' => (string)$row['code'],
            ];
        }

        return $this->responder->ok($data);
    }

    /**
     * 通用字典查询：/common/dict/{code}。
     *
     * - 仅返回启用状态的字典项
     * - 返回格式：[{ label, value, extra(颜色) }]
     */
    private function listDictByCode(ServerRequestInterface $request, array $args): ResponseInterface
    {
        $code = trim((string)($args['code'] ?? ''));
        if ($code === '') {
            return $this->responder->ok([]);
        }

        $sql = "
SELECT di.label,
       di.value,
       COALESCE(di.color, '') AS extra
FROM sys_dict_item AS di
LEFT JOIN sys_dict AS d ON di.dict_id = d.id
WHERE di.status = 1
  AND d.code = :code
ORDER BY di.sort ASC, di.id ASC";

        try {
            $stmt = $this->db->prepare($sql);
            $stmt->execute([':code' => $code]);
            $rows = $stmt->fetchAll();
        } catch (PDOException $e) {
            return $this->responder->fail('500', '查询字典失败');
        }

        $data = [];
        foreach ($rows as $row) {
            $data[] = [
                'label' => (string)$row['label'],
                'value' => (string)$row['value'],
                'extra' => (string)$row['extra'],
            ];
        }

        return $this->responder->ok($data);
    }
}

