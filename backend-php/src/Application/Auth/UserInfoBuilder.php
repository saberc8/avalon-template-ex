<?php

declare(strict_types=1);

namespace Voc\Admin\Application\Auth;

use Voc\Admin\Domain\User\User;

/**
 * 用户信息构造器，返回前端所需的 UserInfo 结构。
 */
final class UserInfoBuilder
{
    /**
     * @param string[] $roleCodes
     * @param string[] $permissions
     */
    public static function build(User $user, array $roleCodes, array $permissions, string $deptName = '', bool $pwdExpired = false): array
    {
        $pwdResetTime = $user->pwdResetTime ? $user->pwdResetTime->format('Y-m-d H:i:s') : '';
        $registrationDate = $user->createTime->format('Y-m-d');

        return [
            'id'               => $user->id,
            'username'         => $user->username,
            'nickname'         => $user->nickname,
            'gender'           => $user->gender,
            'email'            => $user->email ?? '',
            'phone'            => $user->phone ?? '',
            'avatar'           => $user->avatar ?? '',
            'description'      => $user->description ?? '',
            'pwdResetTime'     => $pwdResetTime,
            'pwdExpired'       => $pwdExpired,
            'registrationDate' => $registrationDate,
            'deptName'         => $deptName,
            'roles'            => array_values($roleCodes),
            'permissions'      => array_values($permissions),
        ];
    }

    /**
     * 将角色列表提取为 code 数组。
     */
    public static function extractRoleCodes(array $roles): array
    {
        $codes = [];
        foreach ($roles as $role) {
            /** @var \Voc\Admin\Domain\Rbac\Role $role */
            $codes[] = $role->code;
        }
        return $codes;
    }
}

