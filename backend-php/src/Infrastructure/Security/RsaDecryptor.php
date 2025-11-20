<?php

declare(strict_types=1);

namespace Voc\Admin\Infrastructure\Security;

/**
 * 与 Java / Go 版 SecureUtils / RSADecryptor 兼容的 RSA 解密器。
 * 入参为 Base64 编码的 PKCS#8 私钥（不含 PEM 头尾），以及 Base64 密文。
 */
final class RsaDecryptor
{
    /** @var resource|\OpenSSLAsymmetricKey|null */
    private $privateKey;

    public function __construct(string $base64Pkcs8Key)
    {
        if ($base64Pkcs8Key === '') {
            throw new \InvalidArgumentException('RSA 私钥不能为空');
        }

        $pem = $this->toPem($base64Pkcs8Key);
        $key = openssl_pkey_get_private($pem);
        if ($key === false) {
            throw new \RuntimeException('解析 RSA 私钥失败');
        }

        $this->privateKey = $key;
    }

    /**
     * 解密前端传入的 Base64 密文，返回明文字符串。
     */
    public function decryptBase64(string $cipherBase64): string
    {
        if ($cipherBase64 === '') {
            throw new \InvalidArgumentException('密文不能为空');
        }

        $cipher = base64_decode($cipherBase64, true);
        if ($cipher === false) {
            throw new \RuntimeException('Base64 密文解析失败');
        }

        $ok = openssl_private_decrypt($cipher, $plain, $this->privateKey, OPENSSL_PKCS1_PADDING);
        if (!$ok) {
            throw new \RuntimeException('RSA 解密失败');
        }

        return $plain;
    }

    /**
     * 将纯 Base64 的 PKCS#8 私钥包装成 PEM 形式，供 openssl 使用。
     */
    private function toPem(string $base64Pkcs8Key): string
    {
        $b64 = trim($base64Pkcs8Key);
        $b64 = chunk_split($b64, 64, "\n");
        return "-----BEGIN PRIVATE KEY-----\n" . $b64 . "-----END PRIVATE KEY-----\n";
    }
}

