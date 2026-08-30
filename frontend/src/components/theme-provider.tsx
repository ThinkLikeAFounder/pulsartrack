"use client";

import { useEffect } from "react";
import { useThemeStore } from "@/store/theme-store";

function resolveTheme(theme: "light" | "dark" | "system") {
  if (theme === "system") {
    return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
  }
  return theme;
}

export function ThemeProvider({ children }: { children: React.ReactNode }) {
  const theme = useThemeStore((state) => state.theme);

  useEffect(() => {
    const media = window.matchMedia("(prefers-color-scheme: dark)");

    const applyTheme = () => {
      const activeTheme = resolveTheme(theme);
      document.documentElement.classList.toggle("dark", activeTheme === "dark");
    };

    applyTheme();

    if (theme !== "system") {
      return;
    }

    const onChange = () => applyTheme();
    media.addEventListener("change", onChange);
    return () => media.removeEventListener("change", onChange);
  }, [theme]);

  return <>{children}</>;
}
