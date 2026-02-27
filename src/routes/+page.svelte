<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-dialog';
  import { onMount } from 'svelte';
  import RefSelector from '$lib/RefSelector.svelte';
  import DiffViewer from '$lib/DiffViewer.svelte';
  import CommentBox from '$lib/CommentBox.svelte';
  import QueueStatus from '$lib/QueueStatus.svelte';
  import { initTheme, setTheme, getPreference } from '$lib/theme.svelte';

  onMount(() => initTheme());

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
  let viewMode = $state<'split' | 'unified'>('split');

  // Line selection state for comment box
  let selectionFile = $state('');
  let selectionStart = $state(0);
  let selectionEnd = $state(0);
  let selectionContext = $state('');
  let showCommentBox = $state(false);

  function handleLineSelect(file: string, startLine: number, endLine: number, codeContext: string) {
    selectionFile = file;
    selectionStart = startLine;
    selectionEnd = endLine;
    selectionContext = codeContext;
    showCommentBox = true;
  }

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
    <QueueStatus />
    <button onclick={() => viewMode = viewMode === 'split' ? 'unified' : 'split'}>
      {viewMode === 'split' ? 'Unified' : 'Split'}
    </button>
    <select value={getPreference()} onchange={(e) => setTheme(e.currentTarget.value as 'light' | 'dark' | 'system')}>
      <option value="system">System</option>
      <option value="light">Light</option>
      <option value="dark">Dark</option>
    </select>
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
          <DiffViewer
            file={selectedFile}
            {viewMode}
            onLineSelect={handleLineSelect}
          />
        {:else}
          <p class="empty">Select a file to view diff</p>
        {/if}
      </section>
    {:else}
      <p class="empty">Open a repo and select two refs to compare.</p>
    {/if}
  </div>

  {#if showCommentBox}
    <CommentBox
      file={selectionFile}
      startLine={selectionStart}
      endLine={selectionEnd}
      codeContext={selectionContext}
      onSubmit={() => { showCommentBox = false; }}
      onCancel={() => { showCommentBox = false; }}
    />
  {/if}
</main>

<style>
  :global(:root) {
    --bg-primary: #1a1a1a;
    --bg-secondary: #1e1e1e;
    --bg-header: #252525;
    --bg-hover: #2a2a2a;
    --bg-active: #333;
    --bg-input: #1e1e1e;
    --bg-button: #3a3a3a;
    --bg-button-hover: #4a4a4a;
    --border: #333;
    --border-light: #444;
    --border-medium: #555;
    --text-primary: #e0e0e0;
    --text-secondary: #aaa;
    --text-muted: #666;
    --text-dimmed: #888;
    --text-file: #ccc;
    --text-white: #fff;
    --line-num: #555;
    --line-num-hover: #aaa;
    --hunk-bg: #1e3a5f;
    --hunk-text: #79b8ff;
    --diff-add-bg: #12261e;
    --diff-delete-bg: #2d1517;
    --diff-context-bg: #1a1a1a;
    --selected-bg: #264f78;
    --added-badge-bg: #2ea04333;
    --added-badge-text: #3fb950;
    --modified-badge-bg: #d2992233;
    --modified-badge-text: #d29922;
    --deleted-badge-bg: #f8514933;
    --deleted-badge-text: #f85149;
    --submit-bg: #347d39;
    --submit-hover: #3e8e41;
    --queue-pending-text: #d29922;
    --queue-pending-bg: #d2992215;
    --shadow: rgba(0, 0, 0, 0.5);
  }
  :global([data-theme="light"]) {
    --bg-primary: #ffffff;
    --bg-secondary: #f6f6f6;
    --bg-header: #f0f0f0;
    --bg-hover: #e8e8e8;
    --bg-active: #ddd;
    --bg-input: #fff;
    --bg-button: #e0e0e0;
    --bg-button-hover: #d0d0d0;
    --border: #ddd;
    --border-light: #ccc;
    --border-medium: #bbb;
    --text-primary: #1a1a1a;
    --text-secondary: #555;
    --text-muted: #999;
    --text-dimmed: #777;
    --text-file: #333;
    --text-white: #000;
    --line-num: #999;
    --line-num-hover: #333;
    --hunk-bg: #ddf4ff;
    --hunk-text: #0969da;
    --diff-add-bg: #dafbe1;
    --diff-delete-bg: #ffebe9;
    --diff-context-bg: #ffffff;
    --selected-bg: #b6d4fe;
    --added-badge-bg: #2ea04333;
    --added-badge-text: #1a7f37;
    --modified-badge-bg: #d2992233;
    --modified-badge-text: #9a6700;
    --deleted-badge-bg: #f8514933;
    --deleted-badge-text: #cf222e;
    --submit-bg: #2da44e;
    --submit-hover: #218838;
    --queue-pending-text: #9a6700;
    --queue-pending-bg: #d2992225;
    --shadow: rgba(0, 0, 0, 0.15);
  }
  :global(body) {
    margin: 0;
    background: var(--bg-primary);
    color: var(--text-primary);
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
    background: var(--bg-header);
    border-bottom: 1px solid var(--border);
  }
  header button {
    background: var(--bg-button);
    color: var(--text-primary);
    border: 1px solid var(--border-medium);
    border-radius: 4px;
    padding: 4px 12px;
    cursor: pointer;
    font-size: 13px;
  }
  header button:hover { background: var(--bg-button-hover); }
  .repo-path {
    font-size: 12px;
    color: var(--text-dimmed);
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
    border-right: 1px solid var(--border);
    overflow-y: auto;
    padding: 4px 0;
    background: var(--bg-secondary);
  }
  .file-entry {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 6px 12px;
    background: none;
    border: none;
    color: var(--text-file);
    font-size: 13px;
    cursor: pointer;
    text-align: left;
  }
  .file-entry:hover { background: var(--bg-hover); }
  .file-entry.active { background: var(--bg-active); color: var(--text-white); }
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
  :global(.status-badge.added) { background: var(--added-badge-bg); color: var(--added-badge-text); }
  :global(.status-badge.modified) { background: var(--modified-badge-bg); color: var(--modified-badge-text); }
  :global(.status-badge.deleted) { background: var(--deleted-badge-bg); color: var(--deleted-badge-text); }
  .diff-pane {
    flex: 1;
    overflow: auto;
    padding: 16px;
  }
  .empty {
    color: var(--text-muted);
    text-align: center;
    margin-top: 40px;
  }
  h3 {
    margin: 0 0 12px;
    font-size: 14px;
    color: var(--text-secondary);
  }
</style>
