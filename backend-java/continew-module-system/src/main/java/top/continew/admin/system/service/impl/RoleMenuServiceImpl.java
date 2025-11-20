

package top.continew.admin.system.service.impl;

import cn.hutool.core.collection.CollUtil;
import lombok.RequiredArgsConstructor;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;
import top.continew.admin.system.mapper.RoleMenuMapper;
import top.continew.admin.system.model.entity.RoleMenuDO;
import top.continew.admin.system.service.RoleMenuService;

import java.util.ArrayList;
import java.util.List;
import java.util.stream.Collectors;

/**
 * 角色和菜单业务实现
 *
 * @author Charles7c
 * @since 2023/2/19 10:43
 */
@Service
@RequiredArgsConstructor
public class RoleMenuServiceImpl implements RoleMenuService {

    private final RoleMenuMapper baseMapper;

    @Override
    @Transactional(rollbackFor = Exception.class)
    public boolean add(List<Long> menuIds, Long roleId) {
        // 检查是否有变更
        List<Long> oldMenuIdList = baseMapper.lambdaQuery()
            .select(RoleMenuDO::getMenuId)
            .eq(RoleMenuDO::getRoleId, roleId)
            .list()
            .stream()
            .map(RoleMenuDO::getMenuId)
            .collect(Collectors.toList());
        if (CollUtil.isEmpty(CollUtil.disjunction(menuIds, oldMenuIdList))) {
            return false;
        }
        // 删除原有关联
        baseMapper.lambdaUpdate().eq(RoleMenuDO::getRoleId, roleId).remove();
        // 保存最新关联
        List<RoleMenuDO> roleMenuList = menuIds.stream().map(menuId -> new RoleMenuDO(roleId, menuId)).toList();
        return baseMapper.insertBatch(roleMenuList);
    }

    @Override
    @Transactional(rollbackFor = Exception.class)
    public void deleteByRoleIds(List<Long> roleIds) {
        if (CollUtil.isEmpty(roleIds)) {
            return;
        }
        baseMapper.lambdaUpdate().in(RoleMenuDO::getRoleId, roleIds).remove();
    }

    @Override
    public List<Long> listMenuIdByRoleIds(List<Long> roleIds) {
        if (CollUtil.isEmpty(roleIds)) {
            return new ArrayList<>(0);
        }
        return baseMapper.selectMenuIdByRoleIds(roleIds);
    }
}
