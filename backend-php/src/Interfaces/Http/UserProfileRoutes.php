<?php

declare(strict_types=1);

namespace Voc\Admin\Interfaces\Http;

use PDO;
use PDOException;
use Psr\Http\Message\ResponseInterface;
use Psr\Http\Message\ServerRequestInterface;
use Psr\Http\Message\UploadedFileInterface;
use Slim\App;
use Voc\Admin\Domain\User\UserRepository;
use Voc\Admin\Infrastructure\Response\JsonResponder;
use Voc\Admin\Infrastructure\Security\PasswordService;
use Voc\Admin\Infrastructure\Security\RsaDecryptor;
use Voc\Admin\Infrastructure\Security\TokenService;

/**
 * 个人信息相关路由：/user/profile 系列接口。
 *
 * 对齐 Java 版 UserProfileController 和前端 /user/profile 接口约定，
 * 实现头像上传、基础信息修改、密码/手机号/邮箱修改等能力。
 */
final class UserProfileRoutes
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

    /**
     * 注册 /user/profile 相关路由。
     */
    public function register(App $app): void
    {
        // 上传头像
        $app->patch('/user/profile/avatar', fn (ServerRequestInterface $r) => $this->updateAvatar($r));

        // 修改基础信息
        $app->patch('/user/profile/basic/info', fn (ServerRequestInterface $r) => $this->updateBasicInfo($r));

        // 修改密码
        $app->patch('/user/profile/password', fn (ServerRequestInterface $r) => $this->updatePassword($r));

        // 修改手机号
        $app->patch('/user/profile/phone', fn (ServerRequestInterface $r) => $this->updatePhone($r));

        // 修改邮箱
        $app->patch('/user/profile/email', fn (ServerRequestInterface $r) => $this->updateEmail($r));

        // 三方账号相关接口：当前暂不持久化，仅为了兼容前端调用。
        $app->get('/user/profile/social', fn (ServerRequestInterface $r) => $this->listSocial($r));
        $app->post('/user/profile/social/{source}', fn (ServerRequestInterface $r, array $a) => $this->bindSocial($r, $a));
        $app->delete('/user/profile/social/{source}', fn (ServerRequestInterface $r, array $a) => $this->unbindSocial($r, $a));
    }

    /**
     * 从 Authorization 解析当前登录用户 ID。
     */
    private function currentUserId(ServerRequestInterface $request): int
    {
        $authz = $request->getHeaderLine('Authorization');
        try {
            $claims = $this->tokens->parse($authz);
        } catch (\Throwable $e) {
            throw new \RuntimeException('未授权，请重新登录');
        }
        $uid = (int)($claims['userId'] ?? 0);
        if ($uid <= 0) {
            throw new \RuntimeException('未授权，请重新登录');
        }
        return $uid;
    }

    /**
     * 上传头像：PATCH /user/profile/avatar。
     *
     * 前端通过 FormData 传入字段 avatarFile。
     * 返回：{ avatar: string }，对应用户信息中的 avatar 字段。
     */
    private function updateAvatar(ServerRequestInterface $request): ResponseInterface
    {
        try {
            $userId = $this->currentUserId($request);
        } catch (\RuntimeException $e) {
            return $this->responder->fail('401', $e->getMessage());
        }

        $files = $request->getUploadedFiles();
        $file = $files['avatarFile'] ?? null;
        if (!$file instanceof UploadedFileInterface) {
            return $this->responder->fail('400', '头像不能为空');
        }
        if ($file->getError() !== UPLOAD_ERR_OK) {
            return $this->responder->fail('400', '头像上传失败');
        }

        $clientName = $file->getClientFilename() ?? 'avatar.png';
        $ext = strtolower((string)pathinfo($clientName, PATHINFO_EXTENSION));
        if ($ext === '') {
            $ext = 'png';
        }

        $baseDir = dirname(__DIR__, 3) . '/public/uploads/avatar';
        if (!is_dir($baseDir) && !mkdir($baseDir, 0777, true) && !is_dir($baseDir)) {
            return $this->responder->fail('500', '创建头像目录失败');
        }

        $fileName = 'u' . $userId . '_' . (int)(microtime(true) * 1000) . '.' . $ext;
        $targetPath = $baseDir . '/' . $fileName;
        try {
            $file->moveTo($targetPath);
        } catch (\Throwable $e) {
            return $this->responder->fail('500', '保存头像文件失败');
        }

        $avatarUrl = '/uploads/avatar/' . $fileName;
        $now = (new \DateTimeImmutable())->format('Y-m-d H:i:s');

        $sql = "
UPDATE sys_user
   SET avatar      = :avatar,
       update_user = :update_user,
       update_time = :update_time
 WHERE id          = :id";
        $stmt = $this->db->prepare($sql);
        $stmt->execute([
            ':avatar'      => $avatarUrl,
            ':update_user' => $userId,
            ':update_time' => $now,
            ':id'          => $userId,
        ]);

        return $this->responder->ok(['avatar' => $avatarUrl]);
    }

    /**
     * 修改基础信息：PATCH /user/profile/basic/info。
     *
     * body: { nickname: string, gender: number }
     */
    private function updateBasicInfo(ServerRequestInterface $request): ResponseInterface
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

        $nickname = trim((string)($data['nickname'] ?? ''));
        $gender = (int)($data['gender'] ?? 0);

        if ($nickname === '') {
            return $this->responder->fail('400', '昵称不能为空');
        }
        if ($gender < 0 || $gender > 2) {
            $gender = 0;
        }

        $now = (new \DateTimeImmutable())->format('Y-m-d H:i:s');
        $sql = "
UPDATE sys_user
   SET nickname    = :nickname,
       gender      = :gender,
       update_user = :update_user,
       update_time = :update_time
 WHERE id          = :id";
        $stmt = $this->db->prepare($sql);
        $stmt->execute([
            ':nickname'    => $nickname,
            ':gender'      => $gender,
            ':update_user' => $userId,
            ':update_time' => $now,
            ':id'          => $userId,
        ]);

        return $this->responder->ok(true);
    }

    /**
     * 修改密码：PATCH /user/profile/password。
     *
     * body: { oldPassword: string, newPassword: string }
     * 两个字段均为前端 RSA 加密并 Base64 编码后的字符串。
     */
    private function updatePassword(ServerRequestInterface $request): ResponseInterface
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

        $oldEnc = trim((string)($data['oldPassword'] ?? ''));
        $newEnc = trim((string)($data['newPassword'] ?? ''));
        if ($oldEnc === '' || $newEnc === '') {
            return $this->responder->fail('400', '密码不能为空');
        }

        try {
            $rawOld = $this->rsa->decryptBase64($oldEnc);
        } catch (\Throwable $e) {
            return $this->responder->fail('400', '当前密码解密失败');
        }
        try {
            $rawNew = $this->rsa->decryptBase64($newEnc);
        } catch (\Throwable $e) {
            return $this->responder->fail('400', '新密码解密失败');
        }

        if (strlen($rawNew) < 8 || strlen($rawNew) > 32) {
            return $this->responder->fail('400', '密码长度为 8-32 个字符，至少包含字母和数字');
        }
        $hasLetter = preg_match('/[A-Za-z]/', $rawNew) === 1;
        $hasDigit = preg_match('/[0-9]/', $rawNew) === 1;
        if (!$hasLetter || !$hasDigit) {
            return $this->responder->fail('400', '密码长度为 8-32 个字符，至少包含字母和数字');
        }

        $repo = new UserRepository($this->db);
        $user = $repo->findById($userId);
        if ($user === null) {
            return $this->responder->fail('404', '用户不存在');
        }
        if (!$this->passwords->verify($rawOld, $user->password)) {
            return $this->responder->fail('400', '当前密码不正确');
        }

        try {
            $encodedPwd = $this->passwords->hash($rawNew);
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
            ':id'             => $userId,
        ]);

        return $this->responder->ok(true);
    }

    /**
     * 修改手机号：PATCH /user/profile/phone。
     *
     * body: { phone: string, captcha: string, oldPassword: string }
     * 当前实现仅校验旧密码，不对验证码做后端校验。
     */
    private function updatePhone(ServerRequestInterface $request): ResponseInterface
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

        $phone = trim((string)($data['phone'] ?? ''));
        $oldEnc = trim((string)($data['oldPassword'] ?? ''));
        if ($phone === '' || $oldEnc === '') {
            return $this->responder->fail('400', '手机号和密码不能为空');
        }

        try {
            $rawOld = $this->rsa->decryptBase64($oldEnc);
        } catch (\Throwable $e) {
            return $this->responder->fail('400', '当前密码解密失败');
        }

        $repo = new UserRepository($this->db);
        $user = $repo->findById($userId);
        if ($user === null) {
            return $this->responder->fail('404', '用户不存在');
        }
        if (!$this->passwords->verify($rawOld, $user->password)) {
            return $this->responder->fail('400', '当前密码不正确');
        }

        $now = (new \DateTimeImmutable())->format('Y-m-d H:i:s');
        $sql = "
UPDATE sys_user
   SET phone       = :phone,
       update_user = :update_user,
       update_time = :update_time
 WHERE id          = :id";
        $stmt = $this->db->prepare($sql);
        $stmt->execute([
            ':phone'       => $phone,
            ':update_user' => $userId,
            ':update_time' => $now,
            ':id'          => $userId,
        ]);

        return $this->responder->ok(true);
    }

    /**
     * 修改邮箱：PATCH /user/profile/email。
     *
     * body: { email: string, captcha: string, oldPassword: string }
     * 验证码当前不依赖缓存系统，仅做参数存在性检查。
     */
    private function updateEmail(ServerRequestInterface $request): ResponseInterface
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

        $email = trim((string)($data['email'] ?? ''));
        $captcha = trim((string)($data['captcha'] ?? ''));
        $oldEnc = trim((string)($data['oldPassword'] ?? ''));
        if ($email === '' || $oldEnc === '') {
            return $this->responder->fail('400', '邮箱和密码不能为空');
        }
        if ($captcha === '') {
            return $this->responder->fail('400', '验证码不能为空');
        }

        try {
            $rawOld = $this->rsa->decryptBase64($oldEnc);
        } catch (\Throwable $e) {
            return $this->responder->fail('400', '当前密码解密失败');
        }

        $repo = new UserRepository($this->db);
        $user = $repo->findById($userId);
        if ($user === null) {
            return $this->responder->fail('404', '用户不存在');
        }
        if (!$this->passwords->verify($rawOld, $user->password)) {
            return $this->responder->fail('400', '当前密码不正确');
        }

        $now = (new \DateTimeImmutable())->format('Y-m-d H:i:s');
        $sql = "
UPDATE sys_user
   SET email       = :email,
       update_user = :update_user,
       update_time = :update_time
 WHERE id          = :id";
        $stmt = $this->db->prepare($sql);
        $stmt->execute([
            ':email'       => $email,
            ':update_user' => $userId,
            ':update_time' => $now,
            ':id'          => $userId,
        ]);

        return $this->responder->ok(true);
    }

    /**
     * 获取绑定的三方账号列表：GET /user/profile/social。
     *
     * 当前未实现三方登录，统一返回空数组，避免前端报错。
     */
    private function listSocial(ServerRequestInterface $request): ResponseInterface
    {
        try {
            $this->currentUserId($request);
        } catch (\RuntimeException $e) {
            return $this->responder->fail('401', $e->getMessage());
        }
        return $this->responder->ok([]);
    }

    /**
     * 绑定三方账号：POST /user/profile/social/{source}。
     *
     * 当前仅校验登录状态，直接返回成功。
     */
    private function bindSocial(ServerRequestInterface $request, array $args): ResponseInterface
    {
        try {
            $this->currentUserId($request);
        } catch (\RuntimeException $e) {
            return $this->responder->fail('401', $e->getMessage());
        }
        return $this->responder->ok(true);
    }

    /**
     * 解绑三方账号：DELETE /user/profile/social/{source}。
     *
     * 当前仅校验登录状态，直接返回成功。
     */
    private function unbindSocial(ServerRequestInterface $request, array $args): ResponseInterface
    {
        try {
            $this->currentUserId($request);
        } catch (\RuntimeException $e) {
            return $this->responder->fail('401', $e->getMessage());
        }
        return $this->responder->ok(true);
    }
}

