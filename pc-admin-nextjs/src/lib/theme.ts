export const APPEARANCE_STORAGE_KEY = "avalon-admin-appearance";

export const APPEARANCE_STYLES = ["default", "compact", "spacious"] as const;
export const PRIMARY_COLORS = [
  "black",
  "red",
  "orange",
  "amber",
  "yellow",
  "lime",
  "green",
  "emerald",
  "teal",
  "cyan",
  "sky",
  "blue",
  "indigo",
  "violet",
  "purple",
  "fuchsia",
  "pink",
  "rose"
] as const;
export const NEUTRAL_COLORS = ["neutral", "slate", "zinc", "stone"] as const;
export const ICON_COLORS = ["inherit", "primary", "accent", "muted"] as const;
export const RADIUS_OPTIONS = ["none", "sm", "md", "lg"] as const;

export type AppearanceStyle = (typeof APPEARANCE_STYLES)[number];
export type PrimaryColor = (typeof PRIMARY_COLORS)[number];
export type NeutralColor = (typeof NEUTRAL_COLORS)[number];
export type IconColor = (typeof ICON_COLORS)[number];
export type RadiusOption = (typeof RADIUS_OPTIONS)[number];

export interface AppearanceSettings {
  style: AppearanceStyle;
  primary: PrimaryColor;
  neutral: NeutralColor;
  iconColor: IconColor;
  radius: RadiusOption;
}

export const DEFAULT_APPEARANCE: AppearanceSettings = {
  style: "default",
  primary: "black",
  neutral: "zinc",
  iconColor: "inherit",
  radius: "md"
};

export function serializeAppearance(settings: AppearanceSettings) {
  return JSON.stringify(settings);
}

export function deserializeAppearance(value: string | null): AppearanceSettings {
  if (!value) {
    return DEFAULT_APPEARANCE;
  }

  try {
    const parsed = JSON.parse(value) as Partial<Record<keyof AppearanceSettings, unknown>>;
    return normalizeAppearance(parsed);
  } catch {
    return DEFAULT_APPEARANCE;
  }
}

export function normalizeAppearance(
  value: Partial<Record<keyof AppearanceSettings, unknown>>
): AppearanceSettings {
  return {
    style: pickOption(APPEARANCE_STYLES, value.style, DEFAULT_APPEARANCE.style),
    primary: pickOption(PRIMARY_COLORS, value.primary, DEFAULT_APPEARANCE.primary),
    neutral: pickOption(NEUTRAL_COLORS, value.neutral, DEFAULT_APPEARANCE.neutral),
    iconColor: pickOption(ICON_COLORS, value.iconColor, DEFAULT_APPEARANCE.iconColor),
    radius: pickOption(RADIUS_OPTIONS, value.radius, DEFAULT_APPEARANCE.radius)
  };
}

export function appearanceAttributes(settings: AppearanceSettings) {
  return {
    "data-ui-style": settings.style,
    "data-primary": settings.primary,
    "data-neutral": settings.neutral,
    "data-icon-color": settings.iconColor,
    "data-radius": settings.radius
  };
}

function pickOption<T extends string>(options: readonly T[], value: unknown, fallback: T): T {
  return typeof value === "string" && options.includes(value as T) ? (value as T) : fallback;
}
