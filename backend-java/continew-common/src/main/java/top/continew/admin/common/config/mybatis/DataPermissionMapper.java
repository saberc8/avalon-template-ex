

package top.continew.admin.common.config.mybatis;

import com.baomidou.mybatisplus.core.conditions.Wrapper;
import com.baomidou.mybatisplus.core.metadata.IPage;
import com.baomidou.mybatisplus.core.toolkit.Constants;
import org.apache.ibatis.annotations.Param;
import top.continew.starter.data.mp.base.BaseMapper;
import top.continew.starter.extension.datapermission.annotation.DataPermission;

import java.util.List;

/**
 * 数据权限 Mapper 基类
 *
 * @param <T> 实体类
 * @author Charles7c
 * @since 2023/9/3 21:50
 */
public interface DataPermissionMapper<T> extends BaseMapper<T> {

    /**
     * 根据 entity 条件，查询全部记录
     *
     * @param queryWrapper 实体对象封装操作类（可以为 null）
     * @return 全部记录
     */
    @DataPermission
    @Override
    List<T> selectList(@Param(Constants.WRAPPER) Wrapper<T> queryWrapper);

    /**
     * 根据 entity 条件，查询全部记录（并翻页）
     *
     * @param page         分页查询条件
     * @param queryWrapper 实体对象封装操作类（可以为 null）
     * @return 全部记录（并翻页）
     */
    @DataPermission
    @Override
    List<T> selectList(IPage<T> page, @Param(Constants.WRAPPER) Wrapper<T> queryWrapper);

    /**
     * 根据 ID 删除
     *
     * @param obj     主键ID或实体
     * @param useFill 是否填充
     * @return 删除个数
     */
    @DataPermission
    @Override
    int deleteById(Object obj, boolean useFill);
}
