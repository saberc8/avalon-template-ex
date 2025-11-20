

package top.continew.admin.system.mapper;

import com.baomidou.mybatisplus.core.conditions.query.QueryWrapper;
import com.baomidou.mybatisplus.core.metadata.IPage;
import com.baomidou.mybatisplus.core.toolkit.Constants;
import org.apache.ibatis.annotations.Param;
import top.continew.admin.system.model.entity.LogDO;
import top.continew.admin.system.model.resp.log.LogResp;
import top.continew.starter.data.mp.base.BaseMapper;

import java.util.List;

/**
 * 系统日志 Mapper
 *
 * @author Charles7c
 * @since 2022/12/22 21:47
 */
public interface LogMapper extends BaseMapper<LogDO> {

    /**
     * 分页查询列表
     *
     * @param page         分页条件
     * @param queryWrapper 查询条件
     * @return 分页列表信息
     */
    IPage<LogResp> selectLogPage(@Param("page") IPage<LogDO> page,
                                 @Param(Constants.WRAPPER) QueryWrapper<LogDO> queryWrapper);

    /**
     * 查询列表
     *
     * @param queryWrapper 查询条件
     * @return 列表信息
     */
    List<LogResp> selectLogList(@Param(Constants.WRAPPER) QueryWrapper<LogDO> queryWrapper);

}
