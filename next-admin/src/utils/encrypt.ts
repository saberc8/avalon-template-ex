import { JSEncrypt } from "jsencrypt";

const publicKey =
  "MFwwDQYJKoZIhvcNAQEBBQADSwAwSAJBAM51dgYtMyF+tTQt80sfFOpSV27a7t9u" +
  "aUVeFrdGiVxscuizE7H8SMntYqfn9lp8a5GH5P1/GGehVjUD2gF/4kcCAwEAAQ==";

export function encryptByRsa(text: string): string | false {
  const encryptor = new JSEncrypt();
  encryptor.setPublicKey(publicKey);
  return encryptor.encrypt(text);
}

