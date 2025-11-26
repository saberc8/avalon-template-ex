from passlib.hash import bcrypt


def _truncate_to_72_bytes(raw_password: str) -> str:
    """将明文密码按 BCrypt 规范截断到 72 字节，保持与 Java/Go 行为一致。"""
    if not raw_password:
        return raw_password
    data = raw_password.encode("utf-8")
    if len(data) <= 72:
        return raw_password
    truncated = data[:72]
    return truncated.decode("utf-8", errors="ignore")


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
        # 先按 BCrypt 规范将明文截断到 72 字节，兼容 Java/Go 的实现
        raw = _truncate_to_72_bytes(raw_password)
        enc = encoded_password
        if enc.startswith("{bcrypt}"):
            enc = enc[len("{bcrypt}") :]
        return bcrypt.verify(raw, enc)


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
        # 与校验逻辑一致，先截断到 72 字节再加密
        raw = _truncate_to_72_bytes(raw_password)
        hashed = bcrypt.hash(raw)
        return "{bcrypt}" + hashed
