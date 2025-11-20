

package top.continew.admin.auth.handler;

import jakarta.servlet.http.HttpServletRequest;
import org.springframework.stereotype.Component;
import top.continew.admin.auth.AbstractLoginHandler;
import top.continew.admin.auth.enums.AuthTypeEnum;
import top.continew.admin.auth.model.req.EmailLoginReq;
import top.continew.admin.auth.model.resp.LoginResp;
import top.continew.admin.common.constant.CacheConstants;
import top.continew.admin.system.model.entity.user.UserDO;
import top.continew.admin.system.model.resp.ClientResp;
import top.continew.starter.cache.redisson.util.RedisUtils;
import top.continew.starter.core.validation.ValidationUtils;

/**
 * 邮箱登录处理器
 *
 * @author KAI
 * @author Charles7c
 * @since 2024/12/22 14:58
 */
@Component
public class EmailLoginHandler extends AbstractLoginHandler<EmailLoginReq> {

    @Override
    public LoginResp login(EmailLoginReq req, ClientResp client, HttpServletRequest request) {
        // 验证邮箱
        UserDO user = userService.getByEmail(req.getEmail());
        ValidationUtils.throwIfNull(user, "此邮箱未绑定本系统账号");
        // 检查用户状态
        super.checkUserStatus(user);
        // 执行认证
        String token = super.authenticate(user, client);
        return LoginResp.builder().token(token).build();
    }

    @Override
    public void preLogin(EmailLoginReq req, ClientResp client, HttpServletRequest request) {
        String email = req.getEmail();
        String captchaKey = CacheConstants.CAPTCHA_KEY_PREFIX + email;
        String captcha = RedisUtils.get(captchaKey);
        ValidationUtils.throwIfBlank(captcha, CAPTCHA_EXPIRED);
        ValidationUtils.throwIfNotEqualIgnoreCase(req.getCaptcha(), captcha, CAPTCHA_ERROR);
        RedisUtils.delete(captchaKey);
    }

    @Override
    public AuthTypeEnum getAuthType() {
        return AuthTypeEnum.EMAIL;
    }
}