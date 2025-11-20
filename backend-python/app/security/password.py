from passlib.hash import bcrypt


class PasswordVerifier:
    """BCrypt 密码校验器，兼容 `{bcrypt}` 前缀格式。"""

    @staticmethod
    def verify(raw_password: str, encoded_password: str) -> bool:
        if not encoded_password:
            return False
        enc = encoded_password
        if enc.startswith("{bcrypt}"):
            enc = enc[len("{bcrypt}") :]
        return bcrypt.verify(raw_password, enc)


