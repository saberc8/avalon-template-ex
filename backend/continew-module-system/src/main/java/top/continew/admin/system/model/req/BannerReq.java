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

package top.continew.admin.system.model.req;

import io.swagger.v3.oas.annotations.media.Schema;
import jakarta.validation.constraints.NotBlank;
import jakarta.validation.constraints.NotNull;
import lombok.Data;
import org.hibernate.validator.constraints.Length;
import top.continew.admin.common.enums.DisEnableStatusEnum;

import java.io.Serial;
import java.io.Serializable;

/**
 * Banner 创建或修改请求参数
 *
 * @author Generated
 * @since 2025/11/18
 */
@Data
@Schema(description = "Banner 创建或修改请求参数")
public class BannerReq implements Serializable {

    @Serial
    private static final long serialVersionUID = 1L;

    /**
     * 标题
     */
    @Schema(description = "标题", example = "首页 Banner")
    @NotBlank(message = "标题不能为空")
    @Length(max = 100, message = "标题长度不能超过 {max} 个字符")
    private String title;

    /**
     * 图片地址
     */
    @Schema(description = "图片地址", example = "https://example.com/banner.png")
    @NotBlank(message = "图片地址不能为空")
    @Length(max = 255, message = "图片地址长度不能超过 {max} 个字符")
    private String imageUrl;

    /**
     * 跳转链接
     */
    @Schema(description = "跳转链接", example = "https://example.com")
    @Length(max = 255, message = "跳转链接长度不能超过 {max} 个字符")
    private String linkUrl;

    /**
     * 排序
     */
    @Schema(description = "排序", example = "1")
    private Integer sort;

    /**
     * 状态
     */
    @Schema(description = "状态", example = "1")
    @NotNull(message = "状态不能为空")
    private DisEnableStatusEnum status;

    /**
     * 备注
     */
    @Schema(description = "备注", example = "首页顶部 Banner")
    @Length(max = 255, message = "备注长度不能超过 {max} 个字符")
    private String remark;
}
