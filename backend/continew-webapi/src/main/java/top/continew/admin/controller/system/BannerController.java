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

package top.continew.admin.controller.system;

import io.swagger.v3.oas.annotations.tags.Tag;
import org.springframework.validation.annotation.Validated;
import org.springframework.web.bind.annotation.RestController;
import top.continew.admin.common.controller.BaseController;
import top.continew.admin.system.model.query.BannerQuery;
import top.continew.admin.system.model.req.BannerReq;
import top.continew.admin.system.model.resp.BannerResp;
import top.continew.admin.system.service.BannerService;
import top.continew.starter.extension.crud.annotation.CrudRequestMapping;
import top.continew.starter.extension.crud.enums.Api;

/**
 * Banner 管理 API
 *
 * @author Generated
 * @since 2025/11/18
 */
@Tag(name = "Banner 管理 API")
@Validated
@RestController
@CrudRequestMapping(value = "/system/banner", api = {Api.PAGE, Api.GET, Api.CREATE, Api.UPDATE, Api.DELETE})
public class BannerController extends BaseController<BannerService, BannerResp, BannerResp, BannerQuery, BannerReq> {
}
