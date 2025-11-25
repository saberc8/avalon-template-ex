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
 * 验证码路由：/captcha/image。
 *
 * 仅实现登录图片验证码开关，与 LOGIN_CAPTCHA_ENABLED 配置对齐。
 * 实际图片由 Java/Go 版本提供时，可继续走原有后端。
 */
final class CaptchaRoutes
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
        $app->get('/captcha/image', fn (ServerRequestInterface $r) => $this->getImageCaptcha($r));
    }

    private function getImageCaptcha(ServerRequestInterface $request): ResponseInterface
    {
        $enabled = false;
        try {
            $sql = "
SELECT COALESCE(value, default_value, '0') AS val
FROM sys_option
WHERE code = 'LOGIN_CAPTCHA_ENABLED'
LIMIT 1";
            $stmt = $this->db->query($sql);
            $row = $stmt->fetch();
            if ($row !== false) {
                $val = trim((string)$row['val']);
                $enabled = $val !== '' && $val !== '0';
            }
        } catch (PDOException $e) {
            return $this->responder->fail('500', '查询登录验证码配置失败');
        }

        $expireMs = (int)(microtime(true) * 1000) + 2 * 60 * 1000;

        if (!$enabled) {
            return $this->responder->ok([
                'uuid'       => '',
                'img'        => '',
                'expireTime' => $expireMs,
                'isEnabled'  => false,
            ]);
        }

        // 这里不生成真实图片，仅声明启用，方便前端根据 isEnabled 控制是否展示验证码。
        return $this->responder->ok([
            'uuid'       => '',
            'img'        => '',
            'expireTime' => $expireMs,
            'isEnabled'  => true,
        ]);
    }
}

