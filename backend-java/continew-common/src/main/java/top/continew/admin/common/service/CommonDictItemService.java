

package top.continew.admin.common.service;

import top.continew.starter.extension.crud.model.resp.LabelValueResp;

import java.util.List;

/**
 * 公共字典项业务接口
 *
 * @author Charles7c
 * @since 2025/4/9 20:17
 */
public interface CommonDictItemService {

    /**
     * 根据字典编码查询
     *
     * @param dictCode 字典编码
     * @return 字典项列表
     */
    List<LabelValueResp> listByDictCode(String dictCode);
}
