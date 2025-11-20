<?php

declare(strict_types=1);

namespace Voc\Admin\Infrastructure\Response;

use Psr\Http\Message\ResponseInterface;
use Slim\Psr7\Response;

/**
 * 统一封装响应结构，严格对齐 Go 版 APIResponse：
 * { code, data, msg, success, timestamp }
 */
final class JsonResponder
{
    /**
     * 成功响应包装。
     *
     * @param mixed $data
     */
    public function ok($data): ResponseInterface
    {
        $payload = [
            'code'      => '200',
            'data'      => $data,
            'msg'       => '操作成功',
            'success'   => true,
            'timestamp' => $this->nowMillis(),
        ];

        $response = new Response();
        $response->getBody()->write(json_encode($payload, JSON_UNESCAPED_UNICODE));
        return $response->withHeader('Content-Type', 'application/json; charset=utf-8');
    }

    /**
     * 失败响应包装。
     */
    public function fail(string $code, string $msg): ResponseInterface
    {
        $payload = [
            'code'      => $code,
            'data'      => null,
            'msg'       => $msg,
            'success'   => false,
            'timestamp' => $this->nowMillis(),
        ];

        $response = new Response();
        $response->getBody()->write(json_encode($payload, JSON_UNESCAPED_UNICODE));
        return $response->withHeader('Content-Type', 'application/json; charset=utf-8');
    }

    /**
     * 生成当前时间戳（毫秒字符串），前端使用 Number(res.timestamp)。
     */
    private function nowMillis(): string
    {
        $micro = microtime(true);
        return sprintf('%.0f', $micro * 1000);
    }
}

