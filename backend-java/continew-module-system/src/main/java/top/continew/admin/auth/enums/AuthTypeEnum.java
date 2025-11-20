

package top.continew.admin.auth.enums;

import lombok.Getter;
import lombok.RequiredArgsConstructor;
import top.continew.admin.common.constant.UiConstants;
import top.continew.starter.core.enums.BaseEnum;

/**
 * 认证类型枚举
 *
 * @author KAI
 * @author Charles7c
 * @since 2024/12/22 14:52
 */
@Getter
@RequiredArgsConstructor
public enum AuthTypeEnum implements BaseEnum<String> {

    /**
     * 账号
     */
    ACCOUNT("ACCOUNT", "账号", UiConstants.COLOR_SUCCESS),

    /**
     * 邮箱
     */
    EMAIL("EMAIL", "邮箱", UiConstants.COLOR_PRIMARY),

    /**
     * 手机号
     */
    PHONE("PHONE", "手机号", UiConstants.COLOR_PRIMARY),

    /**
     * 第三方账号
     */
    SOCIAL("SOCIAL", "第三方账号", UiConstants.COLOR_ERROR);

    private final String value;
    private final String description;
    private final String color;
}
