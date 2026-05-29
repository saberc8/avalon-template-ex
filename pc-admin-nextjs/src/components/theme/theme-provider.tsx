"use client";

import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode
} from "react";
import {
  APPEARANCE_STORAGE_KEY,
  DEFAULT_APPEARANCE,
  appearanceAttributes,
  deserializeAppearance,
  serializeAppearance,
  type AppearanceSettings
} from "@/lib/theme";

interface AppearanceContextValue {
  appearance: AppearanceSettings;
  setAppearance: (next: Partial<AppearanceSettings>) => void;
  resetAppearance: () => void;
}

const AppearanceContext = createContext<AppearanceContextValue | null>(null);

export function ThemeProvider({ children }: { children: ReactNode }) {
  const [appearance, setAppearanceState] = useState<AppearanceSettings>(DEFAULT_APPEARANCE);
  const [loaded, setLoaded] = useState(false);

  useEffect(() => {
    setAppearanceState(deserializeAppearance(window.localStorage.getItem(APPEARANCE_STORAGE_KEY)));
    setLoaded(true);
  }, []);

  useEffect(() => {
    applyAppearance(appearance);
    if (loaded) {
      window.localStorage.setItem(APPEARANCE_STORAGE_KEY, serializeAppearance(appearance));
    }
  }, [appearance, loaded]);

  const setAppearance = useCallback((next: Partial<AppearanceSettings>) => {
    setAppearanceState((current) => ({ ...current, ...next }));
  }, []);

  const resetAppearance = useCallback(() => {
    setAppearanceState(DEFAULT_APPEARANCE);
  }, []);

  const value = useMemo(
    () => ({ appearance, setAppearance, resetAppearance }),
    [appearance, resetAppearance, setAppearance]
  );

  return <AppearanceContext.Provider value={value}>{children}</AppearanceContext.Provider>;
}

export function useAppearance() {
  const context = useContext(AppearanceContext);
  if (!context) {
    throw new Error("useAppearance must be used within ThemeProvider");
  }
  return context;
}

function applyAppearance(settings: AppearanceSettings) {
  const root = document.documentElement;
  const attrs = appearanceAttributes(settings);
  for (const [key, value] of Object.entries(attrs)) {
    root.setAttribute(key, value);
  }
}
