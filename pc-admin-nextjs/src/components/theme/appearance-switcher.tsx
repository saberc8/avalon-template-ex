"use client";

import { Paintbrush, RotateCcw } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger
} from "@/components/ui/dropdown-menu";
import { useAppearance } from "@/components/theme/theme-provider";
import {
  APPEARANCE_STYLES,
  ICON_COLORS,
  NEUTRAL_COLORS,
  PRIMARY_COLORS,
  RADIUS_OPTIONS,
  type AppearanceSettings
} from "@/lib/theme";

const STYLE_LABELS: Record<AppearanceSettings["style"], string> = {
  default: "默认",
  compact: "紧凑",
  spacious: "宽松"
};

const PRIMARY_LABELS: Record<AppearanceSettings["primary"], string> = {
  black: "黑色",
  red: "红色",
  orange: "橙色",
  yellow: "黄色",
  lime: "青柠",
  green: "绿色",
  teal: "蓝绿",
  cyan: "青色",
  sky: "天蓝",
  blue: "蓝色",
  indigo: "靛蓝",
  emerald: "翠绿",
  violet: "紫罗兰",
  purple: "深紫",
  fuchsia: "洋红",
  pink: "粉色",
  amber: "琥珀",
  rose: "玫红"
};

const NEUTRAL_LABELS: Record<AppearanceSettings["neutral"], string> = {
  neutral: "中性",
  slate: "蓝灰",
  zinc: "冷灰",
  stone: "暖灰"
};

const ICON_LABELS: Record<AppearanceSettings["iconColor"], string> = {
  inherit: "跟随文字",
  primary: "主色",
  accent: "强调色",
  muted: "弱化"
};

const RADIUS_LABELS: Record<AppearanceSettings["radius"], string> = {
  none: "无",
  sm: "小",
  md: "中",
  lg: "大"
};

const PRIMARY_SWATCHES: Record<AppearanceSettings["primary"], string> = {
  black: "bg-zinc-950",
  red: "bg-red-500",
  orange: "bg-orange-500",
  yellow: "bg-yellow-500",
  lime: "bg-lime-500",
  green: "bg-green-500",
  teal: "bg-teal-500",
  cyan: "bg-cyan-500",
  sky: "bg-sky-500",
  blue: "bg-blue-500",
  indigo: "bg-indigo-500",
  emerald: "bg-emerald-500",
  violet: "bg-violet-500",
  purple: "bg-purple-500",
  fuchsia: "bg-fuchsia-500",
  pink: "bg-pink-500",
  amber: "bg-amber-500",
  rose: "bg-rose-500"
};

export function AppearanceSwitcher() {
  const { appearance, setAppearance, resetAppearance } = useAppearance();

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button size="icon" variant="ghost" aria-label="外观设置">
          <Paintbrush className="app-icon" />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="max-h-[calc(100vh-4rem)] w-80 overflow-y-auto">
        <DropdownMenuLabel>
          <div>外观设置</div>
          <div className="text-xs font-normal text-muted-foreground">后台风格会保存在当前浏览器</div>
        </DropdownMenuLabel>
        <DropdownMenuSeparator />
        <RadioSection
          label="后台风格"
          value={appearance.style}
          options={APPEARANCE_STYLES}
          labels={STYLE_LABELS}
          onValueChange={(style) => setAppearance({ style })}
        />
        <RadioSection
          label="基础色"
          value={appearance.primary}
          options={PRIMARY_COLORS}
          labels={PRIMARY_LABELS}
          swatch
          onValueChange={(primary) => setAppearance({ primary })}
        />
        <RadioSection
          label="中性色"
          value={appearance.neutral}
          options={NEUTRAL_COLORS}
          labels={NEUTRAL_LABELS}
          onValueChange={(neutral) => setAppearance({ neutral })}
        />
        <RadioSection
          label="图标颜色"
          value={appearance.iconColor}
          options={ICON_COLORS}
          labels={ICON_LABELS}
          onValueChange={(iconColor) => setAppearance({ iconColor })}
        />
        <RadioSection
          label="圆角"
          value={appearance.radius}
          options={RADIUS_OPTIONS}
          labels={RADIUS_LABELS}
          onValueChange={(radius) => setAppearance({ radius })}
        />
        <DropdownMenuSeparator />
        <DropdownMenuGroup>
          <DropdownMenuItem onClick={resetAppearance}>
            <RotateCcw className="size-4" />
            恢复默认
          </DropdownMenuItem>
        </DropdownMenuGroup>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

function RadioSection<T extends string>({
  label,
  value,
  options,
  labels,
  swatch = false,
  onValueChange
}: {
  label: string;
  value: T;
  options: readonly T[];
  labels: Record<T, string>;
  swatch?: boolean;
  onValueChange: (value: T) => void;
}) {
  return (
    <div className="px-1 py-1">
      <div className="px-1 pb-1 text-xs font-medium text-muted-foreground">{label}</div>
      <DropdownMenuRadioGroup
        value={value}
        className="grid grid-cols-3 gap-1"
        onValueChange={(next) => onValueChange(next as T)}
      >
        {options.map((option) => (
          <DropdownMenuRadioItem
            key={option}
            value={option}
            className="h-8 gap-1.5 rounded-md pl-7 pr-2 text-xs data-[state=checked]:bg-muted"
          >
            {swatch ? (
              <span
                className={`size-3 shrink-0 rounded-full border ${PRIMARY_SWATCHES[option as AppearanceSettings["primary"]]}`}
              />
            ) : null}
            <span className="truncate">{labels[option]}</span>
          </DropdownMenuRadioItem>
        ))}
      </DropdownMenuRadioGroup>
    </div>
  );
}
