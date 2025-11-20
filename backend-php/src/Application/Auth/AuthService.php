<?php

declare(strict_types=1);

namespace Voc\Admin\Application\Auth;

use PDO;
use Voc\Admin\Domain\User\UserRepository;
use Voc\Admin\Infrastructure\Security\PasswordService;
use Voc\Admin\Infrastructure\Security\RsaDecryptor;
use Voc\Admin\Infrastructure\Security\TokenService;

/**
 * 认证应用服务：与 Go 版 Login 逻辑保持一致。
 */
final class AuthService
{
    private UserRepository $users;
    private RsaDecryptor $decryptor;
    private PasswordService $passwords;
    private TokenService $tokens;

    public function __construct(
        PDO $db,
        RsaDecryptor $decryptor,
        PasswordService $passwords,
        TokenService $tokens
    ) {
        $this->users = new UserRepository($db);
        $this->decryptor = $decryptor;
        $this->passwords = $passwords;
        $this->tokens = $tokens;
    }

    /**
     * 登录请求 DTO。
     */
    public function login(array $payload): array
    {
        $clientId = trim((string)($payload['clientId'] ?? ''));
        $authType = strtoupper(trim((string)($payload['authType'] ?? '')));
        $username = trim((string)($payload['username'] ?? ''));
        $passwordEncrypted = trim((string)($payload['password'] ?? ''));

        if ($authType !== '' && $authType !== 'ACCOUNT') {
            throw new \RuntimeException('暂不支持该认证方式');
        }
        if ($clientId === '') {
            throw new \RuntimeException('客户端ID不能为空');
        }
        if ($username === '') {
            throw new \RuntimeException('用户名不能为空');
        }
        if ($passwordEncrypted === '') {
            throw new \RuntimeException('密码不能为空');
        }

        try {
            $rawPassword = $this->decryptor->decryptBase64($passwordEncrypted);
        } catch (\Throwable $e) {
            throw new \RuntimeException('密码解密失败');
        }

        $user = $this->users->findByUsername($username);
        if ($user === null) {
            throw new \RuntimeException('用户名或密码不正确');
        }

        if (!$this->passwords->verify($rawPassword, $user->password)) {
            throw new \RuntimeException('用户名或密码不正确');
        }

        if (!$user->isEnabled()) {
            throw new \RuntimeException('此账号已被禁用，如有疑问，请联系管理员');
        }

        $token = $this->tokens->generate($user->id);
        return ['token' => $token];
    }
}

