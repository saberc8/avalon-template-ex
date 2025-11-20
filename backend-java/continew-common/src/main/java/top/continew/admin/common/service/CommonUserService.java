

package top.continew.admin.common.service;

import cn.crane4j.annotation.ContainerMethod;
import cn.crane4j.annotation.MappingType;
import top.continew.admin.common.constant.ContainerConstants;

/**
 * 公共用户业务接口
 *
 * @author Charles7c
 * @since 2025/1/9 20:17
 */
public interface CommonUserService {

    /**
     * 根据 ID 查询昵称
     *
     * <p>
     * 数据填充容器 {@link ContainerConstants#USER_NICKNAME}
     * </p>
     * 
     * @param id ID
     * @return 昵称
     */
    @ContainerMethod(namespace = ContainerConstants.USER_NICKNAME, type = MappingType.ORDER_OF_KEYS)
    String getNicknameById(Long id);
}
