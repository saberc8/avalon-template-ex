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

package top.continew.admin.system.service.impl;

import org.springframework.stereotype.Service;
import top.continew.admin.system.mapper.BannerMapper;
import top.continew.admin.system.model.entity.BannerDO;
import top.continew.admin.system.model.query.BannerQuery;
import top.continew.admin.system.model.req.BannerReq;
import top.continew.admin.system.model.resp.BannerResp;
import top.continew.admin.system.service.BannerService;
import top.continew.starter.extension.crud.service.BaseServiceImpl;

/**
 * Banner 业务实现
 *
 * @author Generated
 * @since 2025/11/18
 */
@Service
public class BannerServiceImpl extends BaseServiceImpl<BannerMapper, BannerDO, BannerResp, BannerResp, BannerQuery, BannerReq> implements BannerService {}
