import { describe, expect, it } from "vitest";
import {
  DEFAULT_APPEARANCE,
  deserializeAppearance,
  serializeAppearance,
  type AppearanceSettings
} from "@/lib/theme";

describe("appearance settings", () => {
  it("uses defaults when no persisted settings exist", () => {
    expect(deserializeAppearance(null)).toEqual(DEFAULT_APPEARANCE);
  });

  it("parses persisted valid settings", () => {
    const settings: AppearanceSettings = {
      style: "compact",
      primary: "cyan",
      neutral: "neutral",
      iconColor: "accent",
      radius: "lg"
    };

    expect(deserializeAppearance(serializeAppearance(settings))).toEqual(settings);
  });

  it("falls back invalid values independently", () => {
    const value = JSON.stringify({
      style: "invalid",
      primary: "rose",
      neutral: "invalid",
      iconColor: "primary",
      radius: "huge"
    });

    expect(deserializeAppearance(value)).toEqual({
      ...DEFAULT_APPEARANCE,
      primary: "rose",
      iconColor: "primary"
    });
  });
});
