"use client";

import { Moon, Sun } from "lucide-react";
import { useEffect, useState } from "react";

type Theme = "dark" | "light";

const STORAGE_KEY = "campus-connect-theme";

function applyTheme(theme: Theme) {
  document.documentElement.dataset.theme = theme;
  document.documentElement.style.colorScheme = theme;
}

export function ThemeToggle() {
  const [theme, setTheme] = useState<Theme>("dark");

  useEffect(() => {
    const saved = window.localStorage.getItem(STORAGE_KEY);
    const nextTheme: Theme = saved === "light" ? "light" : "dark";
    setTheme(nextTheme);
    applyTheme(nextTheme);
  }, []);

  function chooseTheme(nextTheme: Theme) {
    setTheme(nextTheme);
    window.localStorage.setItem(STORAGE_KEY, nextTheme);
    applyTheme(nextTheme);
  }

  return (
    <div className="inline-flex h-10 items-center rounded-md border border-[#10252f]/15 bg-white p-1 text-sm font-semibold text-[#10252f] shadow-sm" role="group" aria-label="Theme">
      <button
        type="button"
        onClick={() => chooseTheme("dark")}
        className={theme === "dark" ? "inline-flex h-8 w-8 items-center justify-center rounded bg-[#10252f] text-white" : "inline-flex h-8 w-8 items-center justify-center rounded text-[#10252f]/70"}
        aria-label="Use dark theme"
        aria-pressed={theme === "dark"}
        title="Dark theme"
      >
        <Moon className="h-4 w-4" aria-hidden="true" />
      </button>
      <button
        type="button"
        onClick={() => chooseTheme("light")}
        className={theme === "light" ? "inline-flex h-8 w-8 items-center justify-center rounded bg-[#10252f] text-white" : "inline-flex h-8 w-8 items-center justify-center rounded text-[#10252f]/70"}
        aria-label="Use light theme"
        aria-pressed={theme === "light"}
        title="Light theme"
      >
        <Sun className="h-4 w-4" aria-hidden="true" />
      </button>
    </div>
  );
}
