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

package top.continew.admin.system.model.resp;

import com.alibaba.excel.annotation.ExcelIgnoreUnannotated;
import com.alibaba.excel.annotation.ExcelProperty;
import io.swagger.v3.oas.annotations.media.Schema;
import lombok.Data;
import top.continew.admin.common.enums.DisEnableStatusEnum;
import top.continew.admin.common.model.resp.BaseResp;

import java.io.Serial;

/**
 * Banner 响应参数
 *
 * @author Generated
 * @since 2025/11/18
 */
@Data
@ExcelIgnoreUnannotated
@Schema(description = "Banner 响应参数")
public class BannerResp extends BaseResp {

    @Serial
    private static final long serialVersionUID = 1L;

    /**
     * 标题
     */
    @Schema(description = "标题", example = "首页 Banner")
    @ExcelProperty(value = "标题")
    private String title;

    /**
     * 图片地址
     */
    @Schema(description = "图片地址", example = "https://example.com/banner.png")
    @ExcelProperty(value = "图片地址")
    private String imageUrl;

    /**
     * 跳转链接
     */
    @Schema(description = "跳转链接", example = "https://example.com")
    @ExcelProperty(value = "跳转链接")
    private String linkUrl;

    /**
     * 排序
     */
    @Schema(description = "排序", example = "1")
    @ExcelProperty(value = "排序")
    private Integer sort;

    /**
     * 状态
     */
    @Schema(description = "状态", example = "1")
    @ExcelProperty(value = "状态")
    private DisEnableStatusEnum status;

    /**
     * 备注
     */
    @Schema(description = "备注", example = "首页顶部 Banner")
    @ExcelProperty(value = "备注")
    private String remark;
}
