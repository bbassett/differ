type Theme = 'light' | 'dark';

let current = $state<Theme>(getInitialTheme());

function getInitialTheme(): Theme {
  if (typeof window === 'undefined') return 'dark';
  const saved = localStorage.getItem('theme');
  if (saved === 'light' || saved === 'dark') return saved;
  return window.matchMedia('(prefers-color-scheme: light)').matches ? 'light' : 'dark';
}

function applyTheme(theme: Theme) {
  if (typeof document === 'undefined') return;
  document.documentElement.setAttribute('data-theme', theme);
}

export function initTheme() {
  applyTheme(current);
  window.matchMedia('(prefers-color-scheme: light)').addEventListener('change', (e) => {
    if (!localStorage.getItem('theme')) {
      current = e.matches ? 'light' : 'dark';
      applyTheme(current);
    }
  });
}

export function toggleTheme() {
  current = current === 'dark' ? 'light' : 'dark';
  localStorage.setItem('theme', current);
  applyTheme(current);
}

export function getTheme(): Theme {
  return current;
}
