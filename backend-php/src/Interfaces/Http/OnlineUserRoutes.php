<?php

declare(strict_types=1);

namespace Voc\Admin\Interfaces\Http;

use Psr\Http\Message\ResponseInterface;
use Psr\Http\Message\ServerRequestInterface;
use Slim\App;
use Voc\Admin\Infrastructure\Response\JsonResponder;
use Voc\Admin\Infrastructure\Security\TokenService;

/**
 * 在线用户路由：/monitor/online。
 *
 * 当前 PHP 版本不维护在线会话，仅提供与前端兼容的接口结构：
 * - GET /monitor/online：返回空分页结果
 * - DELETE /monitor/online/{token}：做基本鉴权与自踢校验后直接返回成功
 */
final class OnlineUserRoutes
{
    private TokenService $tokens;
    private JsonResponder $responder;

    public function __construct(TokenService $tokens, JsonResponder $responder)
    {
        $this->tokens = $tokens;
        $this->responder = $responder;
    }

    public function register(App $app): void
    {
        $app->get('/monitor/online', fn (ServerRequestInterface $r) => $this->listOnlineUser($r));
        $app->delete('/monitor/online/{token}', fn (ServerRequestInterface $r, array $a) => $this->kickout($r, $a));
    }

    private function listOnlineUser(ServerRequestInterface $request): ResponseInterface
    {
        return $this->responder->ok([
            'list'  => [],
            'total' => 0,
        ]);
    }

    private function kickout(ServerRequestInterface $request, array $args): ResponseInterface
    {
        $token = trim((string)($args['token'] ?? ''));
        if ($token === '') {
            return $this->responder->fail('400', '令牌不能为空');
        }

        $authz = $request->getHeaderLine('Authorization');
        $raw = trim($authz);
        if (str_starts_with(strtolower($raw), 'bearer ')) {
            $raw = trim(substr($raw, 7));
        }
        $currentToken = $raw;

        if ($currentToken !== '' && $currentToken === $token) {
            return $this->responder->fail('400', '不能强退自己');
        }

        try {
            $this->tokens->parse($authz);
        } catch (\Throwable $e) {
            return $this->responder->fail('401', '未授权，请重新登录');
        }

        return $this->responder->ok(true);
    }
}

