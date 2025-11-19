package security

import (
	"crypto/rsa"
	"crypto/x509"
	"encoding/base64"
	"errors"
	"fmt"
	"math/big"
)

// RSADecryptor holds a parsed RSA private key and can decrypt
// Base64-encoded ciphertexts produced by the front-end.
type RSADecryptor struct {
	priv *rsa.PrivateKey
}

// NewRSADecryptorFromBase64 creates a decryptor from a Base64-encoded
// PKCS#8 private key, compatible with the Java SecureUtils configuration.
func NewRSADecryptorFromBase64(b64Key string) (*RSADecryptor, error) {
	if b64Key == "" {
		return nil, errors.New("rsa private key is empty")
	}
	der, err := base64.StdEncoding.DecodeString(b64Key)
	if err != nil {
		return nil, fmt.Errorf("decode private key: %w", err)
	}
	k, err := x509.ParsePKCS8PrivateKey(der)
	if err != nil {
		return nil, fmt.Errorf("parse pkcs8 private key: %w", err)
	}
	priv, ok := k.(*rsa.PrivateKey)
	if !ok {
		return nil, fmt.Errorf("unexpected private key type %T", k)
	}
	return &RSADecryptor{priv: priv}, nil
}

// DecryptBase64 decrypts a Base64-encoded ciphertext using RSA/PKCS1v15.
// It returns the UTF-8 plaintext password.
func (d *RSADecryptor) DecryptBase64(cipherB64 string) (string, error) {
	if d == nil || d.priv == nil {
		return "", errors.New("rsa decryptor not initialized")
	}
	cipherBytes, err := base64.StdEncoding.DecodeString(cipherB64)
	if err != nil {
		return "", fmt.Errorf("decode cipher text: %w", err)
	}
	plain, err := decryptPKCS1v15Insecure(d.priv, cipherBytes)
	if err != nil {
		return "", fmt.Errorf("rsa decrypt: %w", err)
	}
	return string(plain), nil
}

// decryptPKCS1v15Insecure is a minimal PKCS#1 v1.5 RSA decryption implementation
// that intentionally allows 512-bit keys (Go's crypto/rsa blocks them by default).
// This is only for compatibility with the existing Java/Hutool configuration.
func decryptPKCS1v15Insecure(priv *rsa.PrivateKey, ciphertext []byte) ([]byte, error) {
	k := (priv.N.BitLen() + 7) / 8
	if len(ciphertext) != k {
		return nil, errors.New("rsa: incorrect ciphertext length")
	}

	c := new(big.Int).SetBytes(ciphertext)
	if c.Cmp(priv.N) > 0 {
		return nil, errors.New("rsa: decryption error")
	}

	m := new(big.Int).Exp(c, priv.D, priv.N)
	em := m.Bytes()
	if len(em) < k {
		em = append(make([]byte, k-len(em)), em...)
	}
	// Expect 0x00 || 0x02 || PS || 0x00 || M
	if k < 11 {
		return nil, errors.New("rsa: decryption error")
	}
	if em[0] != 0x00 || em[1] != 0x02 {
		return nil, errors.New("rsa: decryption error")
	}
	// Find 0x00 separator; PS must be at least 8 bytes.
	sep := -1
	for i := 2; i < len(em); i++ {
		if em[i] == 0x00 {
			sep = i
			break
		}
	}
	if sep < 10 {
		return nil, errors.New("rsa: decryption error")
	}
	return em[sep+1:], nil
}

