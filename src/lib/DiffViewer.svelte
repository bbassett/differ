<script lang="ts">
  import type { DiffFile } from './types';

  let { file, viewMode = 'split', onLineSelect = () => {} }: {
    file: DiffFile;
    viewMode: 'split' | 'unified';
    onLineSelect: (file: string, startLine: number, endLine: number, codeContext: string) => void;
  } = $props();

  let selectionStart = $state<number | null>(null);
  let selectionEnd = $state<number | null>(null);
  let selectionSide = $state<'old' | 'new' | null>(null);

  function handleLineClick(lineNum: number, side: 'old' | 'new', e: MouseEvent) {
    if (e.shiftKey && selectionStart !== null && selectionSide === side) {
      selectionEnd = lineNum;
      emitSelection();
    } else {
      selectionStart = lineNum;
      selectionEnd = lineNum;
      selectionSide = side;
      emitSelection();
    }
  }

  function emitSelection() {
    if (selectionStart === null || selectionEnd === null) return;
    const start = Math.min(selectionStart, selectionEnd);
    const end = Math.max(selectionStart, selectionEnd);

    const lines = file.hunks.flatMap(h => h.lines);
    const contextLines = lines.filter(l => {
      const num = selectionSide === 'new' ? l.newNum : l.oldNum;
      return num !== null && num >= start && num <= end;
    });
    const codeContext = contextLines.map(l => l.content).join('\n');

    onLineSelect(file.path, start, end, codeContext);
  }

  function isSelected(lineNum: number | null, side: 'old' | 'new'): boolean {
    if (lineNum === null || selectionStart === null || selectionEnd === null || selectionSide !== side) return false;
    const start = Math.min(selectionStart, selectionEnd);
    const end = Math.max(selectionStart, selectionEnd);
    return lineNum >= start && lineNum <= end;
  }

  export function clearSelection() {
    selectionStart = null;
    selectionEnd = null;
    selectionSide = null;
  }
</script>

{#if viewMode === 'split'}
  <div class="diff-split">
    {#each file.hunks as hunk}
      <div class="hunk-header">
        @@ -{hunk.oldStart},{hunk.oldLines} +{hunk.newStart},{hunk.newLines} @@
      </div>
      <div class="hunk-content-split">
        <div class="side old-side">
          {#each hunk.lines as line}
            {#if line.lineType !== 'add'}
              <div
                class="line {line.lineType}"
                class:selected={isSelected(line.oldNum, 'old')}
              >
                <span
                  class="line-num"
                  role="button"
                  tabindex="0"
                  onclick={(e) => line.oldNum && handleLineClick(line.oldNum, 'old', e)}
                  onkeydown={() => {}}
                >{line.oldNum ?? ''}</span>
                <span class="line-content">{line.content}</span>
              </div>
            {:else}
              <div class="line filler"><span class="line-num"></span><span class="line-content"></span></div>
            {/if}
          {/each}
        </div>
        <div class="side new-side">
          {#each hunk.lines as line}
            {#if line.lineType !== 'delete'}
              <div
                class="line {line.lineType}"
                class:selected={isSelected(line.newNum, 'new')}
              >
                <span
                  class="line-num"
                  role="button"
                  tabindex="0"
                  onclick={(e) => line.newNum && handleLineClick(line.newNum, 'new', e)}
                  onkeydown={() => {}}
                >{line.newNum ?? ''}</span>
                <span class="line-content">{line.content}</span>
              </div>
            {:else}
              <div class="line filler"><span class="line-num"></span><span class="line-content"></span></div>
            {/if}
          {/each}
        </div>
      </div>
    {/each}
  </div>
{:else}
  <div class="diff-unified">
    {#each file.hunks as hunk}
      <div class="hunk-header">
        @@ -{hunk.oldStart},{hunk.oldLines} +{hunk.newStart},{hunk.newLines} @@
      </div>
      {#each hunk.lines as line}
        <div
          class="line {line.lineType}"
          class:selected={isSelected(line.newNum ?? line.oldNum, 'new')}
        >
          <span class="line-num old">{line.oldNum ?? ''}</span>
          <span class="line-num new">{line.newNum ?? ''}</span>
          <span class="line-prefix">{line.lineType === 'add' ? '+' : line.lineType === 'delete' ? '-' : ' '}</span>
          <span
            class="line-content"
            role="button"
            tabindex="0"
            onclick={(e) => {
              const num = line.newNum ?? line.oldNum;
              if (num) handleLineClick(num, 'new', e);
            }}
            onkeydown={() => {}}
          >{line.content}</span>
        </div>
      {/each}
    {/each}
  </div>
{/if}

<style>
  .hunk-header {
    background: var(--hunk-bg);
    color: var(--hunk-text);
    padding: 4px 12px;
    font-size: 12px;
    font-family: monospace;
    border-top: 1px solid var(--border);
  }
  .hunk-content-split {
    display: flex;
  }
  .side {
    flex: 1;
    overflow-x: auto;
  }
  .old-side { border-right: 1px solid var(--border); }
  .line {
    display: flex;
    font-family: 'SF Mono', 'Fira Code', monospace;
    font-size: 13px;
    line-height: 20px;
  }
  .line.add { background: var(--diff-add-bg); }
  .line.delete { background: var(--diff-delete-bg); }
  .line.context { background: var(--diff-context-bg); }
  .line.filler { background: var(--diff-context-bg); opacity: 0.5; }
  .line.selected { background: var(--selected-bg) !important; }
  .line-num {
    min-width: 40px;
    padding: 0 8px;
    text-align: right;
    color: var(--line-num);
    user-select: none;
    cursor: pointer;
    flex-shrink: 0;
  }
  .line-num:hover { color: var(--line-num-hover); background: var(--bg-active); }
  .line-content {
    flex: 1;
    padding: 0 8px;
    white-space: pre;
  }
  .line-prefix {
    width: 16px;
    text-align: center;
    flex-shrink: 0;
    color: var(--text-muted);
  }
  .diff-unified .line-num {
    min-width: 35px;
  }
  .diff-unified .line-num.old { border-right: none; }
  .diff-unified .line-num.new { border-right: 1px solid var(--border); }
</style>
