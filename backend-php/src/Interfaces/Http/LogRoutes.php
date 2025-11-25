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
 * 系统日志路由：/system/log 系列接口。
 */
final class LogRoutes
{
    private PDO $db;
    private JsonResponder $responder;

    public function __construct(PDO $db, JsonResponder $responder)
    {
        $this->db = $db;
        $this->responder = $responder;
    }

    public function register(App $app): void
    {
        $app->get('/system/log', fn (ServerRequestInterface $r) => $this->pageLog($r));
        $app->get('/system/log/{id}', fn (ServerRequestInterface $r, array $a) => $this->getLog($r, $a));
        $app->get('/system/log/export/login', fn (ServerRequestInterface $r) => $this->exportLog($r, true));
        $app->get('/system/log/export/operation', fn (ServerRequestInterface $r) => $this->exportLog($r, false));
    }

    private function pageLog(ServerRequestInterface $request): ResponseInterface
    {
        $params = $request->getQueryParams();
        $page = max(1, (int)($params['page'] ?? 1));
        $size = max(1, (int)($params['size'] ?? 10));

        $description = trim((string)($params['description'] ?? ''));
        $module = trim((string)($params['module'] ?? ''));
        $ip = trim((string)($params['ip'] ?? ''));
        $createUser = trim((string)($params['createUserString'] ?? ''));
        $statusStr = trim((string)($params['status'] ?? ''));
        $statusFilter = $statusStr !== '' ? (int)$statusStr : 0;

        $createTimeRange = $params['createTime'] ?? [];
        $startTime = null;
        $endTime = null;
        if (is_array($createTimeRange) && count($createTimeRange) === 2) {
            $startTime = $createTimeRange[0] ?? null;
            $endTime = $createTimeRange[1] ?? null;
        }

        $baseFrom = "
FROM sys_log AS t1
LEFT JOIN sys_user AS t2 ON t2.id = t1.create_user";

        $where = 'WHERE 1=1';
        $args = [];

        if ($description !== '') {
            $where .= ' AND (t1.description ILIKE :desc OR t1.module ILIKE :desc)';
            $args[':desc'] = '%' . $description . '%';
        }
        if ($module !== '') {
            $where .= ' AND t1.module = :module';
            $args[':module'] = $module;
        }
        if ($ip !== '') {
            $where .= ' AND (t1.ip ILIKE :ip OR t1.address ILIKE :ip)';
            $args[':ip'] = '%' . $ip . '%';
        }
        if ($createUser !== '') {
            $where .= ' AND (t2.username ILIKE :cu OR t2.nickname ILIKE :cu)';
            $args[':cu'] = '%' . $createUser . '%';
        }
        if ($statusFilter !== 0) {
            $where .= ' AND t1.status = :status';
            $args[':status'] = $statusFilter;
        }
        if ($startTime && $endTime) {
            $where .= ' AND t1.create_time BETWEEN :start AND :end';
            $args[':start'] = $startTime;
            $args[':end'] = $endTime;
        }

        $countSql = 'SELECT COUNT(*) ' . $baseFrom . ' ' . $where;
        try {
            $stmt = $this->db->prepare($countSql);
            $stmt->execute($args);
            $total = (int)$stmt->fetchColumn();
        } catch (PDOException $e) {
            return $this->responder->fail('500', '查询日志失败');
        }

        if ($total === 0) {
            return $this->responder->ok(['list' => [], 'total' => 0]);
        }

        $offset = ($page - 1) * $size;

        $sql = "
SELECT t1.id,
       t1.description,
       t1.module,
       COALESCE(t1.time_taken, 0),
       COALESCE(t1.ip, ''),
       COALESCE(t1.address, ''),
       COALESCE(t1.browser, ''),
       COALESCE(t1.os, ''),
       COALESCE(t1.status, 1),
       COALESCE(t1.error_msg, ''),
       t1.create_time,
       COALESCE(t2.nickname, '')
{$baseFrom}
{$where}
ORDER BY t1.create_time DESC, t1.id DESC
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
            return $this->responder->fail('500', '查询日志失败');
        }

        $list = [];
        while ($row = $stmt->fetch()) {
            $list[] = [
                'id'               => (int)$row['id'],
                'description'      => (string)$row['description'],
                'module'           => (string)$row['module'],
                'timeTaken'        => (int)$row['time_taken'],
                'ip'               => (string)$row['ip'],
                'address'          => (string)$row['address'],
                'browser'          => (string)$row['browser'],
                'os'               => (string)$row['os'],
                'status'           => (int)$row['status'],
                'errorMsg'         => (string)$row['error_msg'],
                'createUserString' => (string)$row['nickname'],
                'createTime'       => (string)$row['create_time'],
            ];
        }

        return $this->responder->ok(['list' => $list, 'total' => $total]);
    }

    private function getLog(ServerRequestInterface $request, array $args): ResponseInterface
    {
        $id = (int)($args['id'] ?? 0);
        if ($id <= 0) {
            return $this->responder->fail('400', 'ID 参数不正确');
        }

        $sql = "
SELECT t1.id,
       COALESCE(t1.trace_id, ''),
       t1.description,
       t1.module,
       t1.request_url,
       t1.request_method,
       COALESCE(t1.request_headers, ''),
       COALESCE(t1.request_body, ''),
       t1.status_code,
       COALESCE(t1.response_headers, ''),
       COALESCE(t1.response_body, ''),
       COALESCE(t1.time_taken, 0),
       COALESCE(t1.ip, ''),
       COALESCE(t1.address, ''),
       COALESCE(t1.browser, ''),
       COALESCE(t1.os, ''),
       COALESCE(t1.status, 1),
       COALESCE(t1.error_msg, ''),
       t1.create_time,
       COALESCE(t2.nickname, '')
FROM sys_log AS t1
LEFT JOIN sys_user AS t2 ON t2.id = t1.create_user
WHERE t1.id = :id";

        $stmt = $this->db->prepare($sql);
        $stmt->execute([':id' => $id]);
        $row = $stmt->fetch();
        if ($row === false) {
            return $this->responder->fail('404', '日志不存在');
        }

        $resp = [
            'id'               => (int)$row['id'],
            'traceId'          => (string)$row['trace_id'],
            'description'      => (string)$row['description'],
            'module'           => (string)$row['module'],
            'requestUrl'       => (string)$row['request_url'],
            'requestMethod'    => (string)$row['request_method'],
            'requestHeaders'   => (string)$row['request_headers'],
            'requestBody'      => (string)$row['request_body'],
            'statusCode'       => (int)$row['status_code'],
            'responseHeaders'  => (string)$row['response_headers'],
            'responseBody'     => (string)$row['response_body'],
            'timeTaken'        => (int)$row['time_taken'],
            'ip'               => (string)$row['ip'],
            'address'          => (string)$row['address'],
            'browser'          => (string)$row['browser'],
            'os'               => (string)$row['os'],
            'status'           => (int)$row['status'],
            'errorMsg'         => (string)$row['error_msg'],
            'createUserString' => (string)$row['nickname'],
            'createTime'       => (string)$row['create_time'],
        ];

        return $this->responder->ok($resp);
    }

    private function exportLog(ServerRequestInterface $request, bool $isLogin): ResponseInterface
    {
        $params = $request->getQueryParams();
        $description = trim((string)($params['description'] ?? ''));
        $module = trim((string)($params['module'] ?? ''));
        $ip = trim((string)($params['ip'] ?? ''));
        $createUser = trim((string)($params['createUserString'] ?? ''));
        $statusStr = trim((string)($params['status'] ?? ''));
        $statusFilter = $statusStr !== '' ? (int)$statusStr : 0;

        $createTimeRange = $params['createTime'] ?? [];
        $startTime = null;
        $endTime = null;
        if (is_array($createTimeRange) && count($createTimeRange) === 2) {
            $startTime = $createTimeRange[0] ?? null;
            $endTime = $createTimeRange[1] ?? null;
        }

        $baseFrom = "
FROM sys_log AS t1
LEFT JOIN sys_user AS t2 ON t2.id = t1.create_user";

        $where = 'WHERE 1=1';
        $args = [];

        if ($description !== '') {
            $where .= ' AND (t1.description ILIKE :desc OR t1.module ILIKE :desc)';
            $args[':desc'] = '%' . $description . '%';
        }
        if ($module !== '') {
            $where .= ' AND t1.module = :module';
            $args[':module'] = $module;
        }
        if ($ip !== '') {
            $where .= ' AND (t1.ip ILIKE :ip OR t1.address ILIKE :ip)';
            $args[':ip'] = '%' . $ip . '%';
        }
        if ($createUser !== '') {
            $where .= ' AND (t2.username ILIKE :cu OR t2.nickname ILIKE :cu)';
            $args[':cu'] = '%' . $createUser . '%';
        }
        if ($statusFilter !== 0) {
            $where .= ' AND t1.status = :status';
            $args[':status'] = $statusFilter;
        }
        if ($startTime && $endTime) {
            $where .= ' AND t1.create_time BETWEEN :start AND :end';
            $args[':start'] = $startTime;
            $args[':end'] = $endTime;
        }

        $selectSql = "
SELECT t1.id,
       t1.create_time,
       COALESCE(t2.nickname, ''),
       t1.description,
       t1.module,
       COALESCE(t1.status, 1),
       COALESCE(t1.ip, ''),
       COALESCE(t1.address, ''),
       COALESCE(t1.browser, ''),
       COALESCE(t1.os, ''),
       COALESCE(t1.time_taken, 0)";

        $sql = $selectSql . $baseFrom . ' ' . $where . ' ORDER BY t1.create_time DESC, t1.id DESC';

        try {
            $stmt = $this->db->prepare($sql);
            $stmt->execute($args);
        } catch (PDOException $e) {
            return $this->responder->fail('500', '导出日志失败');
        }

        $rows = [];
        while ($row = $stmt->fetch()) {
            $rows[] = $row;
        }

        $response = $this->responder->ok(null);
        $body = $response->getBody();
        $body->rewind();

        if (!$rows) {
            $filename = $isLogin ? 'login-log.csv' : 'operation-log.csv';
            $body->write('');
            return $response
                ->withHeader('Content-Type', 'text/csv; charset=utf-8')
                ->withHeader('Content-Disposition', 'attachment; filename="' . $filename . '"');
        }

        if ($isLogin) {
            $body->write("ID,登录时间,用户昵称,登录行为,状态,登录 IP,登录地点,浏览器,终端系统\n");
            foreach ($rows as $row) {
                $statusText = ((int)$row['status']) === 1 ? '成功' : '失败';
                $line = sprintf(
                    "%d,%s,%s,%s,%s,%s,%s,%s,%s\n",
                    $row['id'],
                    $row['create_time'],
                    $this->escapeCsv((string)$row['nickname']),
                    $this->escapeCsv((string)$row['description']),
                    $statusText,
                    $this->escapeCsv((string)$row['ip']),
                    $this->escapeCsv((string)$row['address']),
                    $this->escapeCsv((string)$row['browser']),
                    $this->escapeCsv((string)$row['os'])
                );
                $body->write($line);
            }
            $filename = 'login-log.csv';
        } else {
            $body->write("ID,操作时间,操作人,操作内容,所属模块,状态,操作 IP,操作地点,耗时（ms）,浏览器,终端系统\n");
            foreach ($rows as $row) {
                $statusText = ((int)$row['status']) === 1 ? '成功' : '失败';
                $line = sprintf(
                    "%d,%s,%s,%s,%s,%s,%s,%s,%d,%s,%s\n",
                    $row['id'],
                    $row['create_time'],
                    $this->escapeCsv((string)$row['nickname']),
                    $this->escapeCsv((string)$row['description']),
                    $this->escapeCsv((string)$row['module']),
                    $statusText,
                    $this->escapeCsv((string)$row['ip']),
                    $this->escapeCsv((string)$row['address']),
                    (int)$row['time_taken'],
                    $this->escapeCsv((string)$row['browser']),
                    $this->escapeCsv((string)$row['os'])
                );
                $body->write($line);
            }
            $filename = 'operation-log.csv';
        }

        return $response
            ->withHeader('Content-Type', 'text/csv; charset=utf-8')
            ->withHeader('Content-Disposition', 'attachment; filename="' . $filename . '"');
    }

    private function escapeCsv(string $val): string
    {
        if ($val === '') {
            return '';
        }
        if (!str_contains($val, ',') &&
            !str_contains($val, '"') &&
            !str_contains($val, "\n") &&
            !str_contains($val, "\r')
        ) {
            return $val;
        }
        $escaped = str_replace('"', '""', $val);
        return '"' . $escaped . '"';
    }
}

