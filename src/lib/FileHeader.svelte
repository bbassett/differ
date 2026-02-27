<script lang="ts">
  import type { DiffFile } from './types';
  import { isViewed } from './viewed.svelte';

  let { file, collapsed, onToggleCollapse, onToggleViewed }: {
    file: DiffFile;
    collapsed: boolean;
    onToggleCollapse: () => void;
    onToggleViewed: () => void;
  } = $props();

  let viewed = $derived(isViewed(file));
</script>

<div class="file-header">
  <button class="file-header-btn" onclick={onToggleCollapse}>
    <span class="collapse-icon">{collapsed ? '▶' : '▼'}</span>
    <span class="status-badge {file.status}">{file.status[0].toUpperCase()}</span>
    <span class="file-path">{file.path}</span>
  </button>
  <button class="viewed-btn" class:viewed onclick={onToggleViewed}>
    {viewed ? '✓ Viewed' : 'Mark viewed'}
  </button>
</div>

<style>
  .file-header {
    position: sticky;
    top: 0;
    z-index: 10;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 12px;
    background: var(--bg-header);
    border-bottom: 1px solid var(--border);
  }
  .file-header-btn {
    display: flex;
    align-items: center;
    gap: 8px;
    background: none;
    border: none;
    color: var(--text-primary);
    font-size: 13px;
    cursor: pointer;
    padding: 0;
  }
  .file-header-btn:hover { color: var(--text-white); }
  .collapse-icon {
    font-size: 10px;
    width: 14px;
    color: var(--text-muted);
  }
  .file-path {
    font-family: 'SF Mono', 'Fira Code', monospace;
    font-size: 13px;
  }
  .viewed-btn {
    background: none;
    border: 1px solid var(--border-medium);
    border-radius: 4px;
    padding: 2px 10px;
    font-size: 12px;
    cursor: pointer;
    color: var(--text-secondary);
  }
  .viewed-btn:hover {
    background: var(--bg-hover);
  }
  .viewed-btn.viewed {
    color: var(--viewed-check);
    border-color: var(--viewed-check);
  }
</style>
