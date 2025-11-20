import { Injectable } from '@nestjs/common';

/**
 * RSA 解密服务，采用与 Python 版本相同的 PKCS#1 v1.5 手工解密方式。
 * 这里直接使用大数运算，兼容 512 位密钥。
 */
@Injectable()
export class RSADecryptor {
  private readonly n: bigint;
  private readonly d: bigint;

  constructor() {
    const keyB64 =
      process.env.AUTH_RSA_PRIVATE_KEY ||
      'MIIBVQIBADANBgkqhkiG9w0BAQEFAASCAT8wggE7AgEAAkEAznV2Bi0zIX61NC3zSx8U6lJXbtru325pRV4Wt0aJXGxy6LMTsfxIye1ip+f2WnxrkYfk/X8YZ6FWNQPaAX/iRwIDAQABAkEAk/VcAusrpIqA5Ac2P5Tj0VX3cOuXmyouaVcXonr7f+6y2YTjLQuAnkcfKKocQI/juIRQBFQIqqW/m1nmz1wGeQIhAO8XaA/KxzOIgU0l/4lm0A2Wne6RokJ9HLs1YpOzIUmVAiEA3Q9DQrpAlIuiT1yWAGSxA9RxcjUM/1kdVLTkv0avXWsCIE0X8woEjK7lOSwzMG6RpEx9YHdopjViOj1zPVH61KTxAiBmv/dlhqkJ4rV46fIXELZur0pj6WC3N7a4brR8a+CLLQIhAMQyerWl2cPNVtE/8tkziHKbwW3ZUiBXU24wFxedT9iV';
    if (!keyB64) {
      throw new Error('AUTH_RSA_PRIVATE_KEY 未配置');
    }
    const der = Buffer.from(keyB64, 'base64');
    // 使用最简单的 ASN.1 解析方式：假定为 PKCS#8 私钥，仅提取 n 与 d。
    // 为避免引入重量级依赖，这里采用非常简化的解析逻辑：
    // 先查找 0x02（INTEGER）序列，第二个 INTEGER 为 n，第五个 INTEGER 为 d。
    const ints: bigint[] = [];
    for (let i = 0; i < der.length; ) {
      if (der[i] !== 0x02) {
        i += 1;
        continue;
      }
      const lenByte = der[i + 1];
      let len: number;
      let offset: number;
      if (lenByte & 0x80) {
        const bytesLen = lenByte & 0x7f;
        len = 0;
        for (let j = 0; j < bytesLen; j++) {
          len = (len << 8) | der[i + 2 + j];
        }
        offset = i + 2 + bytesLen;
      } else {
        len = lenByte;
        offset = i + 2;
      }
      const intBytes = der.slice(offset, offset + len);
      // 去掉前导 0
      let start = 0;
      while (start < intBytes.length && intBytes[start] === 0) {
        start++;
      }
      let value = BigInt(0);
      for (let k = start; k < intBytes.length; k++) {
        value = (value << BigInt(8)) + BigInt(intBytes[k]);
      }
      ints.push(value);
      i = offset + len;
      if (ints.length >= 6) break;
    }
    if (ints.length < 6) {
      throw new Error('RSA 私钥解析失败');
    }
    // 对于 PKCS#8 PrivateKeyInfo，整数顺序通常为：
    // 0: version, 1: n, 2: e, 3: d, 4: p, 5: q ...
    this.n = ints[1];
    this.d = ints[3];
  }

  /**
   * 解密前端 RSA 加密的 Base64 密文。
   */
  decryptBase64(cipherB64: string): string {
    if (!cipherB64) {
      throw new Error('cipher text is empty');
    }
    const cipherBytes = Buffer.from(cipherB64, 'base64');
    const plainBytes = this.decryptPkcs1v15(cipherBytes);
    return plainBytes.toString('utf8');
  }

  /**
   * PKCS#1 v1.5 低强度解密实现，与 Python/Go 版对齐。
   */
  private decryptPkcs1v15(cipher: Buffer): Buffer {
    const k = this.byteLengthOfN();
    if (cipher.length !== k) {
      throw new Error('rsa: incorrect ciphertext length');
    }
    let c = BigInt(0);
    for (const b of cipher.values()) {
      c = (c << BigInt(8)) + BigInt(b);
    }
    if (c >= this.n) {
      throw new Error('rsa: decryption error');
    }
    const m = this.modPow(c, this.d, this.n);
    const em = this.toBytes(m, k);
    if (k < 11) {
      throw new Error('rsa: decryption error');
    }
    if (em[0] !== 0x00 || em[1] !== 0x02) {
      throw new Error('rsa: decryption error');
    }
    let idx = 2;
    for (; idx < em.length; idx++) {
      if (em[idx] === 0x00) break;
    }
    if (idx < 10 || idx >= em.length) {
      throw new Error('rsa: decryption error');
    }
    return em.subarray(idx + 1);
  }

  private byteLengthOfN(): number {
    const bits = this.n.toString(2).length;
    return Math.ceil(bits / 8);
  }

  private modPow(base: bigint, exp: bigint, mod: bigint): bigint {
    let result = BigInt(1);
    let b = base % mod;
    let e = exp;
    while (e > 0) {
      if (e & BigInt(1)) {
        result = (result * b) % mod;
      }
      e >>= BigInt(1);
      b = (b * b) % mod;
    }
    return result;
  }

  private toBytes(num: bigint, length: number): Buffer {
    const bytes = Buffer.alloc(length);
    for (let i = length - 1; i >= 0; i--) {
      bytes[i] = Number(num & BigInt(0xff));
      num >>= BigInt(8);
    }
    return bytes;
  }
}
