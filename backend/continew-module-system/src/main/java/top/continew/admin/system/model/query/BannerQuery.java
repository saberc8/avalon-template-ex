/*
 * Copyright (c) 2022-present Charles7c Authors. All Rights Reserved.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package top.continew.admin.system.model.query;

import io.swagger.v3.oas.annotations.media.Schema;
import lombok.Data;
import top.continew.admin.common.enums.DisEnableStatusEnum;
import top.continew.starter.data.core.annotation.Query;
import top.continew.starter.data.core.enums.QueryType;

import java.io.Serial;
import java.io.Serializable;

/**
 * Banner 查询条件
 *
 * @author Generated
 * @since 2025/11/18
 */
@Data
@Schema(description = "Banner 查询条件")
public class BannerQuery implements Serializable {

    @Serial
    private static final long serialVersionUID = 1L;

    /**
     * 标题关键词
     */
    @Schema(description = "标题关键词", example = "首页")
    @Query(columns = {"title"}, type = QueryType.LIKE)
    private String keyword;

    /**
     * 状态
     */
    @Schema(description = "状态", example = "1")
    @Query
    private DisEnableStatusEnum status;
}
