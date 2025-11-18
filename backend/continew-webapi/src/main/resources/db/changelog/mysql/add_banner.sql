-- liquibase formatted sql

-- changeset codex:banner_table_1
-- comment 新增 Banner 表
CREATE TABLE IF NOT EXISTS `sys_banner` (
    `id`          bigint(20)   NOT NULL AUTO_INCREMENT     COMMENT 'ID',
    `title`       varchar(100) NOT NULL                    COMMENT '标题',
    `image_url`   varchar(255) NOT NULL                    COMMENT '图片地址',
    `link_url`    varchar(255) DEFAULT NULL                COMMENT '跳转链接',
    `sort`        int          NOT NULL DEFAULT 999        COMMENT '排序',
    `status`      tinyint(1)   UNSIGNED NOT NULL DEFAULT 1 COMMENT '状态（1：启用；2：禁用）',
    `remark`      varchar(255) DEFAULT NULL                COMMENT '备注',
    `create_user` bigint(20)   NOT NULL                    COMMENT '创建人',
    `create_time` datetime     NOT NULL                    COMMENT '创建时间',
    `update_user` bigint(20)   DEFAULT NULL                COMMENT '修改人',
    `update_time` datetime     DEFAULT NULL                COMMENT '修改时间',
    PRIMARY KEY (`id`),
    INDEX `idx_create_user`(`create_user`),
    INDEX `idx_update_user`(`update_user`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Banner 表';

