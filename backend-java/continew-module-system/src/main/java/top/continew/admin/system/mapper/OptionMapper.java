

package top.continew.admin.system.mapper;

import org.apache.ibatis.annotations.Param;
import org.apache.ibatis.annotations.Select;
import top.continew.admin.system.model.entity.OptionDO;
import top.continew.starter.data.mp.base.BaseMapper;

import java.util.List;

/**
 * 参数 Mapper
 *
 * @author Bull-BCLS
 * @since 2023/8/26 19:38
 */
public interface OptionMapper extends BaseMapper<OptionDO> {

    /**
     * 根据类别查询
     *
     * @param category 类别
     * @return 列表
     */
    @Select("SELECT code, value, default_value FROM sys_option WHERE category = #{category}")
    List<OptionDO> selectByCategory(@Param("category") String category);
}