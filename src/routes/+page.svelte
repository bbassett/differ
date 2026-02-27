<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-dialog';
  import RefSelector from '$lib/RefSelector.svelte';

  type RefInfo = { name: string; refType: string };
  type DiffResult = {
    baseRef: string;
    compareRef: string;
    files: DiffFile[];
  };
  type DiffFile = {
    path: string;
    status: string;
    oldPath: string | null;
    hunks: DiffHunk[];
  };
  type DiffHunk = {
    oldStart: number;
    oldLines: number;
    newStart: number;
    newLines: number;
    lines: DiffLine[];
  };
  type DiffLine = {
    lineType: string;
    content: string;
    oldNum: number | null;
    newNum: number | null;
  };

  let refs = $state<RefInfo[]>([]);
  let baseRef = $state('');
  let compareRef = $state('');
  let diff = $state<DiffResult | null>(null);
  let selectedFile = $state<DiffFile | null>(null);
  let repoPath = $state('');

  async function openRepo() {
    const selected = await open({ directory: true });
    if (selected) {
      repoPath = selected as string;
      refs = await invoke<RefInfo[]>('open_repo', { path: repoPath });
      const main = refs.find(r => r.name === 'main' || r.name === 'master');
      if (main) baseRef = main.name;
    }
  }

  async function loadDiff() {
    if (!baseRef || !compareRef) return;
    diff = await invoke<DiffResult>('get_diff', { base: baseRef, compare: compareRef });
    if (diff && diff.files.length > 0) {
      selectedFile = diff.files[0];
    }
  }

  $effect(() => {
    if (baseRef && compareRef) loadDiff();
  });
</script>

<main>
  <header>
    <button onclick={openRepo}>Open Repo</button>
    {#if repoPath}
      <span class="repo-path">{repoPath}</span>
    {/if}
    <div class="ref-selectors">
      <RefSelector {refs} bind:selected={baseRef} label="Base" />
      <RefSelector {refs} bind:selected={compareRef} label="Compare" />
    </div>
  </header>

  <div class="workspace">
    {#if diff}
      <aside class="file-tree">
        {#each diff.files as file}
          <button
            class="file-entry"
            class:active={selectedFile === file}
            onclick={() => selectedFile = file}
          >
            <span class="status-badge {file.status}">{file.status[0].toUpperCase()}</span>
            {file.path}
          </button>
        {/each}
      </aside>

      <section class="diff-pane">
        {#if selectedFile}
          <h3>{selectedFile.path}</h3>
          <p>Diff viewer goes here (Task 8)</p>
        {:else}
          <p class="empty">Select a file to view diff</p>
        {/if}
      </section>
    {:else}
      <p class="empty">Open a repo and select two refs to compare.</p>
    {/if}
  </div>
</main>

<style>
  :global(body) {
    margin: 0;
    background: #1a1a1a;
    color: #e0e0e0;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  }
  main {
    display: flex;
    flex-direction: column;
    height: 100vh;
  }
  header {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 12px;
    background: #252525;
    border-bottom: 1px solid #333;
  }
  header button {
    background: #3a3a3a;
    color: #e0e0e0;
    border: 1px solid #555;
    border-radius: 4px;
    padding: 4px 12px;
    cursor: pointer;
    font-size: 13px;
  }
  header button:hover { background: #4a4a4a; }
  .repo-path {
    font-size: 12px;
    color: #888;
    max-width: 200px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .ref-selectors {
    display: flex;
    gap: 16px;
    margin-left: auto;
  }
  .workspace {
    display: flex;
    flex: 1;
    overflow: hidden;
  }
  .file-tree {
    width: 250px;
    border-right: 1px solid #333;
    overflow-y: auto;
    padding: 4px 0;
    background: #1e1e1e;
  }
  .file-entry {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 6px 12px;
    background: none;
    border: none;
    color: #ccc;
    font-size: 13px;
    cursor: pointer;
    text-align: left;
  }
  .file-entry:hover { background: #2a2a2a; }
  .file-entry.active { background: #333; color: #fff; }
  .status-badge {
    font-size: 11px;
    font-weight: 700;
    width: 16px;
    height: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 3px;
  }
  :global(.status-badge.added) { background: #2ea04333; color: #3fb950; }
  :global(.status-badge.modified) { background: #d2992233; color: #d29922; }
  :global(.status-badge.deleted) { background: #f8514933; color: #f85149; }
  .diff-pane {
    flex: 1;
    overflow: auto;
    padding: 16px;
  }
  .empty {
    color: #666;
    text-align: center;
    margin-top: 40px;
  }
  h3 {
    margin: 0 0 12px;
    font-size: 14px;
    color: #aaa;
  }
</style>
