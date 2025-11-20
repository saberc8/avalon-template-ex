

package top.continew.admin.system.model.query;

import io.swagger.v3.oas.annotations.media.Schema;
import lombok.Data;
import top.continew.starter.data.core.annotation.Query;
import top.continew.starter.data.core.enums.QueryType;

import java.io.Serial;
import java.io.Serializable;

/**
 * 角色查询条件
 *
 * @author Charles7c
 * @since 2023/2/8 23:04
 */
@Data
@Schema(description = "角色查询条件")
public class RoleQuery implements Serializable {

    @Serial
    private static final long serialVersionUID = 1L;

    /**
     * 关键词
     */
    @Schema(description = "关键词", example = "测试人员")
    @Query(columns = {"name", "code", "description"}, type = QueryType.LIKE)
    private String description;
}
