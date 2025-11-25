<?php

declare(strict_types=1);

namespace Voc\Admin\Interfaces\Http;

use PDO;
use PDOException;
use Psr\Http\Message\ResponseInterface;
use Psr\Http\Message\ServerRequestInterface;
use Slim\App;
use Voc\Admin\Infrastructure\Response\JsonResponder;
use Voc\Admin\Infrastructure\Security\RsaDecryptor;
use Voc\Admin\Infrastructure\Security\TokenService;

/**
 * 存储配置路由：/system/storage 系列接口。
 */
final class StorageRoutes
{
    private PDO $db;
    private TokenService $tokens;
    private RsaDecryptor $rsa;
    private JsonResponder $responder;

    public function __construct(PDO $db, TokenService $tokens, RsaDecryptor $rsa, JsonResponder $responder)
    {
        $this->db = $db;
        $this->tokens = $tokens;
        $this->rsa = $rsa;
        $this->responder = $responder;
    }

    public function register(App $app): void
    {
        $app->get('/system/storage/list', fn (ServerRequestInterface $r) => $this->listStorage($r));
        $app->get('/system/storage/{id}', fn (ServerRequestInterface $r, array $a) => $this->getStorage($r, $a));
        $app->post('/system/storage', fn (ServerRequestInterface $r) => $this->createStorage($r));
        $app->put('/system/storage/{id}', fn (ServerRequestInterface $r, array $a) => $this->updateStorage($r, $a));
        $app->delete('/system/storage', fn (ServerRequestInterface $r) => $this->deleteStorage($r));
        $app->put('/system/storage/{id}/status', fn (ServerRequestInterface $r, array $a) => $this->updateStorageStatus($r, $a));
        $app->put('/system/storage/{id}/default', fn (ServerRequestInterface $r, array $a) => $this->setDefaultStorage($r, $a));
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

    /**
     * 解密前端传入的密钥，空字符串表示不更新。
     */
    private function decryptSecretKey(?string $encrypted, string $oldVal): string
    {
        $val = trim((string)$encrypted);
        if ($val === '') {
            return $oldVal;
        }
        try {
            $plain = $this->rsa->decryptBase64($val);
        } catch (\Throwable $e) {
            throw new \RuntimeException('私有密钥解密失败');
        }
        if (mb_strlen($plain) > 255) {
            throw new \RuntimeException('私有密钥长度不能超过 255 个字符');
        }
        return $plain;
    }

    private function listStorage(ServerRequestInterface $request): ResponseInterface
    {
        $params = $request->getQueryParams();
        $description = trim((string)($params['description'] ?? ''));
        $typeStr = trim((string)($params['type'] ?? ''));
        $type = $typeStr !== '' ? (int)$typeStr : 0;

        $where = 'WHERE 1=1';
        $args = [];
        if ($description !== '') {
            $where .= ' AND (s.name ILIKE :desc OR s.code ILIKE :desc OR COALESCE(s.description,\'\') ILIKE :desc)';
            $args[':desc'] = '%' . $description . '%';
        }
        if ($type !== 0) {
            $where .= ' AND s.type = :type';
            $args[':type'] = $type;
        }

        $sql = "
SELECT s.id,
       s.name,
       s.code,
       s.type,
       COALESCE(s.access_key, ''),
       COALESCE(s.region, ''),
       COALESCE(s.endpoint, ''),
       s.bucket_name,
       COALESCE(s.domain, ''),
       COALESCE(s.description, ''),
       s.is_default,
       COALESCE(s.sort, 999),
       s.status,
       s.create_time,
       COALESCE(cu.nickname, ''),
       s.update_time,
       COALESCE(uu.nickname, '')
FROM sys_storage AS s
LEFT JOIN sys_user AS cu ON cu.id = s.create_user
LEFT JOIN sys_user AS uu ON uu.id = s.update_user
{$where}
ORDER BY s.sort ASC, s.id ASC";

        try {
            $stmt = $this->db->prepare($sql);
            $stmt->execute($args);
        } catch (PDOException $e) {
            return $this->responder->fail('500', '查询存储配置失败');
        }

        $list = [];
        while ($row = $stmt->fetch()) {
            $list[] = [
                'id'               => (int)$row['id'],
                'name'             => (string)$row['name'],
                'code'             => (string)$row['code'],
                'type'             => (int)$row['type'],
                'accessKey'        => (string)$row['access_key'],
                'secretKey'        => '',
                'endpoint'         => (string)$row['endpoint'],
                'region'           => (string)$row['region'],
                'bucketName'       => (string)$row['bucket_name'],
                'domain'           => (string)$row['domain'],
                'description'      => (string)$row['description'],
                'isDefault'        => (bool)$row['is_default'],
                'sort'             => (int)$row['sort'],
                'status'           => (int)$row['status'],
                'createUserString' => (string)$row['nickname'],
                'createTime'       => (string)$row['create_time'],
                'updateUserString' => (string)$row['nickname'],
                'updateTime'       => $row['update_time'] ? (string)$row['update_time'] : '',
            ];
        }

        return $this->responder->ok($list);
    }

    private function getStorage(ServerRequestInterface $request, array $args): ResponseInterface
    {
        $id = (int)($args['id'] ?? 0);
        if ($id <= 0) {
            return $this->responder->fail('400', 'ID 参数不正确');
        }

        $sql = "
SELECT s.id,
       s.name,
       s.code,
       s.type,
       COALESCE(s.access_key, ''),
       COALESCE(s.secret_key, ''),
       COALESCE(s.endpoint, ''),
       COALESCE(s.region, ''),
       s.bucket_name,
       COALESCE(s.domain, ''),
       COALESCE(s.description, ''),
       s.is_default,
       COALESCE(s.sort, 999),
       s.status,
       s.create_time,
       COALESCE(cu.nickname, ''),
       s.update_time,
       COALESCE(uu.nickname, '')
FROM sys_storage AS s
LEFT JOIN sys_user AS cu ON cu.id = s.create_user
LEFT JOIN sys_user AS uu ON uu.id = s.update_user
WHERE s.id = :id";

        $stmt = $this->db->prepare($sql);
        $stmt->execute([':id' => $id]);
        $row = $stmt->fetch();
        if ($row === false) {
            return $this->responder->fail('404', '存储配置不存在');
        }

        $secret = (string)$row['secret_key'];
        if ($secret !== '') {
            $secret = '******';
        }

        $resp = [
            'id'               => (int)$row['id'],
            'name'             => (string)$row['name'],
            'code'             => (string)$row['code'],
            'type'             => (int)$row['type'],
            'accessKey'        => (string)$row['access_key'],
            'secretKey'        => $secret,
            'endpoint'         => (string)$row['endpoint'],
            'region'           => (string)$row['region'],
            'bucketName'       => (string)$row['bucket_name'],
            'domain'           => (string)$row['domain'],
            'description'      => (string)$row['description'],
            'isDefault'        => (bool)$row['is_default'],
            'sort'             => (int)$row['sort'],
            'status'           => (int)$row['status'],
            'createUserString' => (string)$row['nickname'],
            'createTime'       => (string)$row['create_time'],
            'updateUserString' => (string)$row['nickname'],
            'updateTime'       => $row['update_time'] ? (string)$row['update_time'] : '',
        ];

        return $this->responder->ok($resp);
    }

    private function createStorage(ServerRequestInterface $request): ResponseInterface
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
        $type = (int)($data['type'] ?? 1);
        $accessKey = (string)($data['accessKey'] ?? '');
        $secretKeyEnc = $data['secretKey'] ?? null;
        $endpoint = trim((string)($data['endpoint'] ?? ''));
        $region = trim((string)($data['region'] ?? ''));
        $bucketName = trim((string)($data['bucketName'] ?? ''));
        $domain = trim((string)($data['domain'] ?? ''));
        $description = (string)($data['description'] ?? '');
        $isDefault = (bool)($data['isDefault'] ?? false);
        $sort = (int)($data['sort'] ?? 999);
        $status = (int)($data['status'] ?? 1);

        if ($name === '' || $code === '') {
            return $this->responder->fail('400', '名称和编码不能为空');
        }
        if ($type === 0) {
            $type = 1;
        }
        if ($sort <= 0) {
            $sort = 999;
        }
        if ($status === 0) {
            $status = 1;
        }

        $checkSql = 'SELECT 1 FROM sys_storage WHERE code = :code LIMIT 1';
        $stmt = $this->db->prepare($checkSql);
        $stmt->execute([':code' => $code]);
        if ($stmt->fetch()) {
            return $this->responder->fail('400', '存储编码已存在');
        }

        $secretVal = '';
        if ($type === 2 && is_string($secretKeyEnc)) {
            try {
                $secretVal = $this->decryptSecretKey($secretKeyEnc, '');
            } catch (\RuntimeException $e) {
                return $this->responder->fail('400', $e->getMessage());
            }
        }

        $now = (new \DateTimeImmutable())->format('Y-m-d H:i:s');
        $newId = $this->nextId();

        $sql = "
INSERT INTO sys_storage (
    id, name, code, type, access_key, secret_key, endpoint,
    region, bucket_name, domain, description, is_default, sort, status,
    create_user, create_time
) VALUES (
    :id, :name, :code, :type, :access_key, :secret_key, :endpoint,
    :region, :bucket_name, :domain, :description, :is_default, :sort, :status,
    :create_user, :create_time
)";

        try {
            $stmt = $this->db->prepare($sql);
            $stmt->execute([
                ':id'          => $newId,
                ':name'        => $name,
                ':code'        => $code,
                ':type'        => $type,
                ':access_key'  => $accessKey,
                ':secret_key'  => $secretVal,
                ':endpoint'    => $endpoint,
                ':region'      => $region,
                ':bucket_name' => $bucketName,
                ':domain'      => $domain,
                ':description' => $description,
                ':is_default'  => $isDefault,
                ':sort'        => $sort,
                ':status'      => $status,
                ':create_user' => $userId,
                ':create_time' => $now,
            ]);
        } catch (PDOException $e) {
            return $this->responder->fail('500', '新增存储配置失败');
        }

        return $this->responder->ok(['id' => $newId]);
    }

    private function updateStorage(ServerRequestInterface $request, array $args): ResponseInterface
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
        $type = (int)($data['type'] ?? 1);
        $accessKey = (string)($data['accessKey'] ?? '');
        $secretKeyEnc = array_key_exists('secretKey', $data) ? $data['secretKey'] : null;
        $endpoint = trim((string)($data['endpoint'] ?? ''));
        $region = trim((string)($data['region'] ?? ''));
        $bucketName = trim((string)($data['bucketName'] ?? ''));
        $domain = trim((string)($data['domain'] ?? ''));
        $description = (string)($data['description'] ?? '');
        $sort = (int)($data['sort'] ?? 999);
        $status = (int)($data['status'] ?? 1);

        if ($name === '') {
            return $this->responder->fail('400', '名称不能为空');
        }
        if ($sort <= 0) {
            $sort = 999;
        }
        if ($status === 0) {
            $status = 1;
        }

        $selectOld = 'SELECT COALESCE(secret_key, \'\') AS sk FROM sys_storage WHERE id = :id';
        $stmt = $this->db->prepare($selectOld);
        $stmt->execute([':id' => $id]);
        $row = $stmt->fetch();
        if ($row === false) {
            return $this->responder->fail('404', '存储配置不存在');
        }
        $oldSecret = (string)$row['sk'];

        $now = (new \DateTimeImmutable())->format('Y-m-d H:i:s');

        $params = [
            ':name'        => $name,
            ':type'        => $type,
            ':access_key'  => $accessKey,
            ':endpoint'    => $endpoint,
            ':region'      => $region,
            ':bucket_name' => $bucketName,
            ':domain'      => $domain,
            ':description' => $description,
            ':sort'        => $sort,
            ':status'      => $status,
            ':update_user' => $userId,
            ':update_time' => $now,
            ':id'          => $id,
        ];

        if ($secretKeyEnc !== null && is_string($secretKeyEnc)) {
            try {
                $secretVal = $this->decryptSecretKey($secretKeyEnc, $oldSecret);
            } catch (\RuntimeException $e) {
                return $this->responder->fail('400', $e->getMessage());
            }
            $sql = "
UPDATE sys_storage
   SET name        = :name,
       type        = :type,
       access_key  = :access_key,
       secret_key  = :secret_key,
       endpoint    = :endpoint,
       region      = :region,
       bucket_name = :bucket_name,
       domain      = :domain,
       description = :description,
       sort        = :sort,
       status      = :status,
       update_user = :update_user,
       update_time = :update_time
 WHERE id          = :id";
            $params[':secret_key'] = $secretVal;
        } else {
            $sql = "
UPDATE sys_storage
   SET name        = :name,
       type        = :type,
       access_key  = :access_key,
       endpoint    = :endpoint,
       region      = :region,
       bucket_name = :bucket_name,
       domain      = :domain,
       description = :description,
       sort        = :sort,
       status      = :status,
       update_user = :update_user,
       update_time = :update_time
 WHERE id          = :id";
        }

        try {
            $stmt = $this->db->prepare($sql);
            $stmt->execute($params);
        } catch (PDOException $e) {
            return $this->responder->fail('500', '修改存储配置失败');
        }

        return $this->responder->ok(true);
    }

    private function deleteStorage(ServerRequestInterface $request): ResponseInterface
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
            foreach ($ids as $id) {
                $stmt = $this->db->prepare('SELECT is_default FROM sys_storage WHERE id = :id');
                $stmt->execute([':id' => $id]);
                $row = $stmt->fetch();
                if ($row === false) {
                    continue;
                }
                if ((bool)$row['is_default']) {
                    $this->db->rollBack();
                    return $this->responder->fail('400', '不允许删除默认存储');
                }
                $del = $this->db->prepare('DELETE FROM sys_storage WHERE id = :id');
                $del->execute([':id' => $id]);
            }
            $this->db->commit();
        } catch (PDOException $e) {
            $this->db->rollBack();
            return $this->responder->fail('500', '删除存储配置失败');
        }

        return $this->responder->ok(true);
    }

    private function updateStorageStatus(ServerRequestInterface $request, array $args): ResponseInterface
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
        $status = (int)($data['status'] ?? 0);
        if ($status !== 1 && $status !== 2) {
            return $this->responder->fail('400', '状态参数不正确');
        }

        $stmt = $this->db->prepare('SELECT is_default FROM sys_storage WHERE id = :id');
        $stmt->execute([':id' => $id]);
        $row = $stmt->fetch();
        if ($row === false) {
            return $this->responder->fail('404', '存储配置不存在');
        }
        if ((bool)$row['is_default'] && $status !== 1) {
            return $this->responder->fail('400', '不允许禁用默认存储');
        }

        $now = (new \DateTimeImmutable())->format('Y-m-d H:i:s');

        $sql = "
UPDATE sys_storage
   SET status      = :status,
       update_user = :update_user,
       update_time = :update_time
 WHERE id          = :id";
        try {
            $stmt = $this->db->prepare($sql);
            $stmt->execute([
                ':status'      => $status,
                ':update_user' => $userId,
                ':update_time' => $now,
                ':id'          => $id,
            ]);
        } catch (PDOException $e) {
            return $this->responder->fail('500', '更新存储状态失败');
        }

        return $this->responder->ok(true);
    }

    private function setDefaultStorage(ServerRequestInterface $request, array $args): ResponseInterface
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

        $now = (new \DateTimeImmutable())->format('Y-m-d H:i:s');

        try {
            $this->db->beginTransaction();
            $this->db->exec('UPDATE sys_storage SET is_default = FALSE');

            $sql = "
UPDATE sys_storage
   SET is_default  = TRUE,
       update_user = :update_user,
       update_time = :update_time
 WHERE id          = :id";
            $stmt = $this->db->prepare($sql);
            $stmt->execute([
                ':update_user' => $userId,
                ':update_time' => $now,
                ':id'          => $id,
            ]);

            $this->db->commit();
        } catch (PDOException $e) {
            $this->db->rollBack();
            return $this->responder->fail('500', '设为默认存储失败');
        }

        return $this->responder->ok(true);
    }

    private function nextId(): int
    {
        return (int)(microtime(true) * 1000000) + random_int(1, 1000);
    }
}

