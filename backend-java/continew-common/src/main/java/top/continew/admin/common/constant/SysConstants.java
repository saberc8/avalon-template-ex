

package top.continew.admin.common.constant;

/**
 * 系统相关常量
 *
 * @author Charles7c
 * @since 2023/2/9 22:11
 */
public class SysConstants {

    /**
     * 否
     */
    public static final Integer NO = 0;

    /**
     * 是
     */
    public static final Integer YES = 1;

    /**
     * 超管用户 ID
     */
    public static final Long SUPER_USER_ID = 1L;

    /**
     * 顶级部门 ID
     */
    public static final Long SUPER_DEPT_ID = 1L;

    /**
     * 顶级父 ID
     */
    public static final Long SUPER_PARENT_ID = 0L;

    /**
     * 超管角色编码
     */
    public static final String SUPER_ROLE_CODE = "admin";

    /**
     * 普通用户角色编码
     */
    public static final String GENERAL_ROLE_CODE = "general";

    /**
     * 超管角色 ID
     */
    public static final Long SUPER_ROLE_ID = 1L;

    /**
     * 普通用户角色 ID
     */
    public static final Long GENERAL_ROLE_ID = 2L;

    /**
     * 全部权限标识
     */
    public static final String ALL_PERMISSION = "*:*:*";

    /**
     * 登录 URI
     */
    public static final String LOGIN_URI = "/auth/login";

    /**
     * 登出 URI
     */
    public static final String LOGOUT_URI = "/auth/logout";

    private SysConstants() {
    }
}
