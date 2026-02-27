type Theme = 'light' | 'dark';
type ThemePreference = 'light' | 'dark' | 'system';

let preference = $state<ThemePreference>(getInitialPreference());
let systemTheme = $state<Theme>(getSystemTheme());
let resolved = $derived<Theme>(preference === 'system' ? systemTheme : preference);

function getInitialPreference(): ThemePreference {
  if (typeof window === 'undefined') return 'system';
  const saved = localStorage.getItem('theme');
  if (saved === 'light' || saved === 'dark' || saved === 'system') return saved;
  return 'system';
}

function getSystemTheme(): Theme {
  if (typeof window === 'undefined') return 'dark';
  return window.matchMedia('(prefers-color-scheme: light)').matches ? 'light' : 'dark';
}

function applyTheme(theme: Theme) {
  if (typeof document === 'undefined') return;
  document.documentElement.setAttribute('data-theme', theme);
}

export function initTheme() {
  applyTheme(resolved);
  window.matchMedia('(prefers-color-scheme: light)').addEventListener('change', (e) => {
    systemTheme = e.matches ? 'light' : 'dark';
    applyTheme(resolved);
  });
}

export function setTheme(pref: ThemePreference) {
  preference = pref;
  localStorage.setItem('theme', pref);
  applyTheme(resolved);
}

export function getTheme(): Theme {
  return resolved;
}

export function getPreference(): ThemePreference {
  return preference;
}
