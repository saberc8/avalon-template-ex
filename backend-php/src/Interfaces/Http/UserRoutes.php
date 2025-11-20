<?php

declare(strict_types=1);

namespace Voc\Admin\Interfaces\Http;

use PDO;
use Psr\Http\Message\ResponseInterface;
use Psr\Http\Message\ServerRequestInterface;
use Slim\App;
use Voc\Admin\Application\Auth\RouteBuilder;
use Voc\Admin\Application\Auth\UserInfoBuilder;
use Voc\Admin\Domain\Rbac\MenuRepository;
use Voc\Admin\Domain\Rbac\RoleRepository;
use Voc\Admin\Domain\User\UserRepository;
use Voc\Admin\Infrastructure\Response\JsonResponder;
use Voc\Admin\Infrastructure\Security\TokenService;

/**
 * 用户基础信息与路由接口：/auth/user/info 与 /auth/user/route。
 */
final class UserRoutes
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
        $app->get('/auth/user/info', function (ServerRequestInterface $request): ResponseInterface {
            $authz = $request->getHeaderLine('Authorization');
            try {
                $claims = $this->tokens->parse($authz);
            } catch (\Throwable $e) {
                return $this->responder->fail('401', '未授权，请重新登录');
            }

            $userRepo = new UserRepository($this->db);
            $roleRepo = new RoleRepository($this->db);
            $menuRepo = new MenuRepository($this->db);

            $user = $userRepo->findById($claims['userId']);
            if ($user === null) {
                return $this->responder->fail('401', '未授权，请重新登录');
            }

            $roles = $roleRepo->listByUserId($claims['userId']);
            $roleCodes = UserInfoBuilder::extractRoleCodes($roles);
            $perms = $menuRepo->listPermissionsByUserId($claims['userId']);

            // 暂时不返回部门名称与密码过期标记
            $info = UserInfoBuilder::build($user, $roleCodes, $perms, '', false);
            return $this->responder->ok($info);
        });

        $app->get('/auth/user/route', function (ServerRequestInterface $request): ResponseInterface {
            $authz = $request->getHeaderLine('Authorization');
            try {
                $claims = $this->tokens->parse($authz);
            } catch (\Throwable $e) {
                return $this->responder->fail('401', '未授权，请重新登录');
            }

            $roleRepo = new RoleRepository($this->db);
            $menuRepo = new MenuRepository($this->db);

            $roles = $roleRepo->listByUserId($claims['userId']);
            if (empty($roles)) {
                return $this->responder->ok([]);
            }

            $roleCodes = UserInfoBuilder::extractRoleCodes($roles);
            $menus = $menuRepo->listByUserId($claims['userId']);

            $tree = RouteBuilder::buildRouteTree($menus, $roleCodes);
            return $this->responder->ok($tree);
        });
    }
}

