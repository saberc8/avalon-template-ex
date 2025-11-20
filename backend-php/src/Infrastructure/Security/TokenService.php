<?php

declare(strict_types=1);

namespace Voc\Admin\Infrastructure\Security;

/**
 * 简单 JWT 工具，使用 HS256，与 Go 版 TokenService 行为一致：
 * - 载荷包含 userId, iat, exp
 * - token 为纯字符串，前端通过 Authorization: Bearer <token> 传递。
 */
final class TokenService
{
    private string $secret;
    private int $ttlSeconds;

    public function __construct(string $secret, int $ttlSeconds)
    {
        $this->secret = $secret !== '' ? $secret : 'asdasdasifhueuiwyurfewbfjsdafjk';
        $this->ttlSeconds = $ttlSeconds > 0 ? $ttlSeconds : 24 * 60 * 60;
    }

    /**
     * 生成 token。
     */
    public function generate(int $userId): string
    {
        $now = time();
        $payload = [
            'userId' => $userId,
            'iat'    => $now,
            'exp'    => $now + $this->ttlSeconds,
        ];

        $header = ['alg' => 'HS256', 'typ' => 'JWT'];

        $segments = [
            $this->base64UrlEncode(json_encode($header, JSON_UNESCAPED_UNICODE)),
            $this->base64UrlEncode(json_encode($payload, JSON_UNESCAPED_UNICODE)),
        ];

        $signingInput = implode('.', $segments);
        $signature = hash_hmac('sha256', $signingInput, $this->secret, true);
        $segments[] = $this->base64UrlEncode($signature);

        return implode('.', $segments);
    }

    /**
     * 解析并校验 token，返回载荷数组。
     *
     * @return array{userId:int,iat:int,exp:int}
     */
    public function parse(string $token): array
    {
        $token = trim($token);
        if ($token === '') {
            throw new \RuntimeException('empty token');
        }
        if (stripos($token, 'bearer ') === 0) {
            $token = trim(substr($token, 7));
        }

        $parts = explode('.', $token);
        if (count($parts) !== 3) {
            throw new \RuntimeException('invalid token');
        }

        [$h64, $p64, $s64] = $parts;
        $headerJson = $this->base64UrlDecode($h64);
        $payloadJson = $this->base64UrlDecode($p64);
        $sig = $this->base64UrlDecode($s64);

        if ($headerJson === null || $payloadJson === null || $sig === null) {
            throw new \RuntimeException('invalid token');
        }

        $header = json_decode($headerJson, true);
        $payload = json_decode($payloadJson, true);
        if (!is_array($header) || !is_array($payload)) {
            throw new \RuntimeException('invalid token');
        }
        if (($header['alg'] ?? '') !== 'HS256') {
            throw new \RuntimeException('invalid token');
        }

        $expectedSig = hash_hmac('sha256', $h64 . '.' . $p64, $this->secret, true);
        if (!hash_equals($expectedSig, $sig)) {
            throw new \RuntimeException('invalid token');
        }

        $exp = intval($payload['exp'] ?? 0);
        if ($exp > 0 && $exp < time()) {
            throw new \RuntimeException('token expired');
        }

        return [
            'userId' => intval($payload['userId'] ?? 0),
            'iat'    => intval($payload['iat'] ?? 0),
            'exp'    => $exp,
        ];
    }

    private function base64UrlEncode(string $data): string
    {
        return rtrim(strtr(base64_encode($data), '+/', '-_'), '=');
    }

    private function base64UrlDecode(string $data): ?string
    {
        $remainder = strlen($data) % 4;
        if ($remainder) {
            $data .= str_repeat('=', 4 - $remainder);
        }
        $decoded = base64_decode(strtr($data, '-_', '+/'), true);
        return $decoded === false ? null : $decoded;
    }
}

