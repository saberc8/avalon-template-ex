<?php

declare(strict_types=1);

use Slim\Factory\AppFactory;
use Voc\Admin\Infrastructure\Database;
use Voc\Admin\Infrastructure\Response\JsonResponder;
use Voc\Admin\Infrastructure\Security\RsaDecryptor;
use Voc\Admin\Infrastructure\Security\PasswordService;
use Voc\Admin\Infrastructure\Security\TokenService;
use Voc\Admin\Application\Auth\AuthService;
use Voc\Admin\Interfaces\Http\AuthRoutes;
use Voc\Admin\Interfaces\Http\UserRoutes;
use Voc\Admin\Interfaces\Http\SystemUserRoutes;
use Voc\Admin\Interfaces\Http\RoleRoutes;
use Voc\Admin\Interfaces\Http\MenuRoutes;
use Voc\Admin\Interfaces\Http\DeptRoutes;

require __DIR__ . '/../vendor/autoload.php';

// 加载环境变量（如果存在 .env 文件）
if (file_exists(__DIR__ . '/../.env')) {
    (new Dotenv\Dotenv(__DIR__ . '/..'))->safeLoad();
}

// 初始化 Slim 应用
$app = AppFactory::create();

// 全局错误处理（开发期可打开详细错误）
$app->addErrorMiddleware(true, true, true);

// CORS 中间件，尽可能与 Go 版保持一致
$app->add(function ($request, $handler) {
    $response = $handler->handle($request);
    $origin = $request->getHeaderLine('Origin');

    if ($origin === 'http://localhost:3000') {
        $response = $response
            ->withHeader('Access-Control-Allow-Origin', $origin)
            ->withHeader('Vary', 'Origin');
    }

    return $response
        ->withHeader('Access-Control-Allow-Credentials', 'true')
        ->withHeader('Access-Control-Allow-Headers', 'Content-Type, Authorization, X-Requested-With')
        ->withHeader('Access-Control-Allow-Methods', 'GET, POST, PUT, PATCH, DELETE, OPTIONS');
});

// 预检请求直接返回 204
$app->options('/{routes:.+}', function ($request, $response) {
    return $response->withStatus(204);
});

// 初始化基础组件：数据库、响应包装、安全组件
$db = Database::createFromEnv();
$responder = new JsonResponder();

$rsaKey = getenv('AUTH_RSA_PRIVATE_KEY') ?: 'MIIBVQIBADANBgkqhkiG9w0BAQEFAASCAT8wggE7AgEAAkEAznV2Bi0zIX61NC3zSx8U6lJXbtru325pRV4Wt0aJXGxy6LMTsfxIye1ip+f2WnxrkYfk/X8YZ6FWNQPaAX/iRwIDAQABAkEAk/VcAusrpIqA5Ac2P5Tj0VX3cOuXmyouaVcXonr7f+6y2YTjLQuAnkcfKKocQI/juIRQBFQIqqW/m1nmz1wGeQIhAO8XaA/KxzOIgU0l/4lm0A2Wne6RokJ9HLs1YpOzIUmVAiEA3Q9DQrpAlIuiT1yWAGSxA9RxcjUM/1kdVLTkv0avXWsCIE0X8woEjK7lOSwzMG6RpEx9YHdopjViOj1zPVH61KTxAiBmv/dlhqkJ4rV46fIXELZur0pj6WC3N7a4brR8a+CLLQIhAMQyerWl2cPNVtE/8tkziHKbwW3ZUiBXU24wFxedT9iV';
$rsaDecryptor = new RsaDecryptor($rsaKey);
$passwordService = new PasswordService();

$jwtSecret = getenv('AUTH_JWT_SECRET') ?: 'asdasdasifhueuiwyurfewbfjsdafjk';
$tokenTtlSeconds = 24 * 60 * 60;
$tokenService = new TokenService($jwtSecret, $tokenTtlSeconds);

// 初始化应用服务与路由
$authService = new AuthService($db, $rsaDecryptor, $passwordService, $tokenService);

// 认证相关路由（/auth/login）
(new AuthRoutes($authService, $responder))->register($app);

// 用户与路由相关接口（/auth/user/info、/auth/user/route）
(new UserRoutes($db, $tokenService, $responder))->register($app);

// 系统管理：用户管理 /system/user
(new SystemUserRoutes($db, $tokenService, $rsaDecryptor, $passwordService, $responder))->register($app);

// 系统管理：角色管理 /system/role
(new RoleRoutes($db, $tokenService, $responder))->register($app);

// 系统管理：菜单管理 /system/menu
(new MenuRoutes($db, $tokenService, $responder))->register($app);

// 系统管理：部门管理 /system/dept
(new DeptRoutes($db, $tokenService, $responder))->register($app);

// TODO：后续补充 /system/*、/common/*、/user/profile 等全部接口迁移

// 启动 HTTP 服务
$port = getenv('HTTP_PORT') ?: '4398';
$app->run();
