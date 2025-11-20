

package top.continew.admin.auth.model.resp;

import io.swagger.v3.oas.annotations.media.Schema;
import lombok.Builder;
import lombok.Data;

import java.io.Serial;
import java.io.Serializable;

/**
 * 三方账号授权认证响应参数
 *
 * @author Charles7c
 * @since 2024/3/6 22:26
 */
@Data
@Builder
@Schema(description = "三方账号授权认证响应参数")
public class SocialAuthAuthorizeResp implements Serializable {

    @Serial
    private static final long serialVersionUID = 1L;

    /**
     * 授权 URL
     */
    @Schema(description = "授权 URL", example = "https://gitee.com/oauth/authorize?response_type=code&client_id=5d271b7f638941812aaf8bfc2e2f08f06d6235ef934e0e39537e2364eb8452c4&redirect_uri=http://localhost:5173/social/callback?source=gitee&state=d4ea7129e2531050210e9c918cc007d7&scope=user_info")
    private String authorizeUrl;
}
