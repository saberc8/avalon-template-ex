from passlib.hash import bcrypt


class PasswordVerifier:
    """BCrypt 密码校验器，兼容 `{bcrypt}` 前缀格式。"""

    @staticmethod
    def verify(raw_password: str, encoded_password: str) -> bool:
        """
        校验明文密码与数据库存储密码是否匹配。

        - 支持带有 `{bcrypt}` 前缀的哈希格式；
        - 其余情况直接按 BCrypt 哈希进行校验。
        """
        if not encoded_password:
            return False
        enc = encoded_password
        if enc.startswith("{bcrypt}"):
            enc = enc[len("{bcrypt}") :]
        return bcrypt.verify(raw_password, enc)


class PasswordHasher:
    """BCrypt 加密器，用于生成数据库存储密码哈希。"""

    @staticmethod
    def hash(raw_password: str) -> str:
        """
        生成带 `{bcrypt}` 前缀的 BCrypt 哈希。

        说明：
        - 与 Java 版存储格式保持一致（通常为 `{bcrypt}` 前缀）；
        - 前端仍按原有方式加密，后端仅负责落库加密。
        """
        if not raw_password:
            raise ValueError("密码不能为空")
        hashed = bcrypt.hash(raw_password)
        return "{bcrypt}" + hashed

