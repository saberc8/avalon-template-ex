import base64
from dataclasses import dataclass

from cryptography.hazmat.backends import default_backend
from cryptography.hazmat.primitives import serialization


@dataclass
class RSADecryptor:
    """
    与 Go 版 RSADecryptor 行为保持一致的解密器。

    - 从 Base64 编码的 PKCS#8 私钥派生 n 和 d；
    - 使用手工实现的 PKCS#1 v1.5 解密逻辑，允许 512 位密钥；
    - 入参为 Base64 编码的密文，输出 UTF-8 明文密码。
    """

    n: int
    d: int

    @classmethod
    def from_base64_key(cls, b64_key: str) -> "RSADecryptor":
        if not b64_key:
            raise ValueError("rsa private key is empty")
        der = base64.b64decode(b64_key)
        key = serialization.load_der_private_key(
            der, password=None, backend=default_backend()
        )
        numbers = key.private_numbers()
        n = numbers.public_numbers.n
        d = numbers.d
        return cls(n=n, d=d)

    def decrypt_base64(self, cipher_b64: str) -> str:
        """解密 Base64 编码的密文为 UTF-8 明文。"""
        if not cipher_b64:
            raise ValueError("cipher text is empty")
        cipher_bytes = base64.b64decode(cipher_b64)
        plain = self._decrypt_pkcs1_v15_insecure(cipher_bytes)
        return plain.decode("utf-8")

    def _decrypt_pkcs1_v15_insecure(self, ciphertext: bytes) -> bytes:
        """
        参考 Go 版 decryptPKCS1v15Insecure 的简化实现。

        只做长度与格式基本校验，保持与 Java/Hutool 配置的兼容性。
        """
        k = (self.n.bit_length() + 7) // 8
        if len(ciphertext) != k:
            raise ValueError("rsa: incorrect ciphertext length")

        c = int.from_bytes(ciphertext, "big")
        if c >= self.n:
            raise ValueError("rsa: decryption error")

        m = pow(c, self.d, self.n)
        em = m.to_bytes(k, "big")

        if k < 11:
            raise ValueError("rsa: decryption error")
        if em[0] != 0x00 or em[1] != 0x02:
            raise ValueError("rsa: decryption error")

        try:
            sep = em.index(b"\x00", 2)
        except ValueError as exc:  # pragma: no cover - 理论上不会触发
            raise ValueError("rsa: decryption error") from exc
        if sep < 10:
            raise ValueError("rsa: decryption error")
        return em[sep + 1 :]


