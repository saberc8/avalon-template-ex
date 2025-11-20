<?php

declare(strict_types=1);

namespace Voc\Admin\Interfaces\Http;

use Psr\Http\Message\ResponseInterface;
use Psr\Http\Message\ServerRequestInterface;
use Slim\App;
use Voc\Admin\Application\Auth\AuthService;
use Voc\Admin\Infrastructure\Response\JsonResponder;

/**
 * 认证相关 HTTP 路由：/auth/login。
 */
final class AuthRoutes
{
    private AuthService $auth;
    private JsonResponder $responder;

    public function __construct(AuthService $auth, JsonResponder $responder)
    {
        $this->auth = $auth;
        $this->responder = $responder;
    }

    public function register(App $app): void
    {
        $app->post('/auth/login', function (ServerRequestInterface $request) : ResponseInterface {
            $body = (string)$request->getBody();
            $data = json_decode($body, true);
            if (!is_array($data)) {
                return $this->responder->fail('400', '参数缺失或格式不正确');
            }

            try {
                $resp = $this->auth->login($data);
            } catch (\RuntimeException $e) {
                return $this->responder->fail('400', $e->getMessage());
            } catch (\Throwable $e) {
                return $this->responder->fail('500', '服务器内部错误');
            }

            return $this->responder->ok($resp);
        });
    }
}

