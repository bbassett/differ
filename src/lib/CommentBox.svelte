<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';

  let { file, startLine, endLine, codeContext, onSubmit = () => {}, onCancel = () => {} }: {
    file: string;
    startLine: number;
    endLine: number;
    codeContext: string;
    onSubmit: () => void;
    onCancel: () => void;
  } = $props();

  let comment = $state('');
  let submitting = $state(false);

  async function submit() {
    if (!comment.trim()) return;
    submitting = true;
    try {
      await invoke('submit_comment', {
        file,
        startLine,
        endLine,
        codeContext,
        comment: comment.trim(),
      });
      comment = '';
      onSubmit();
    } catch (e) {
      console.error('Failed to submit comment:', e);
    } finally {
      submitting = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) {
      submit();
    }
    if (e.key === 'Escape') {
      onCancel();
    }
  }
</script>

<div class="comment-box">
  <div class="comment-header">
    <span class="comment-location">{file}:{startLine}{endLine !== startLine ? `-${endLine}` : ''}</span>
    <button class="close-btn" onclick={onCancel}>x</button>
  </div>
  <div class="code-preview">
    <pre>{codeContext}</pre>
  </div>
  <textarea
    bind:value={comment}
    onkeydown={handleKeydown}
    placeholder="Describe what should change..."
    rows="3"
    disabled={submitting}
  ></textarea>
  <div class="comment-actions">
    <span class="hint">Cmd+Enter to send</span>
    <button onclick={submit} disabled={submitting || !comment.trim()}>
      {submitting ? 'Sending...' : 'Send'}
    </button>
  </div>
</div>

<style>
  .comment-box {
    position: fixed;
    bottom: 16px;
    right: 16px;
    width: 400px;
    background: var(--bg-hover);
    border: 1px solid var(--border-medium);
    border-radius: 8px;
    box-shadow: 0 4px 24px var(--shadow);
    z-index: 100;
  }
  .comment-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 12px;
    border-bottom: 1px solid var(--border-light);
  }
  .comment-location {
    font-family: monospace;
    font-size: 12px;
    color: var(--hunk-text);
  }
  .close-btn {
    background: none;
    border: none;
    color: var(--text-dimmed);
    cursor: pointer;
    font-size: 14px;
    padding: 0 4px;
  }
  .close-btn:hover { color: var(--text-white); }
  .code-preview {
    max-height: 80px;
    overflow: auto;
    padding: 8px 12px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-light);
  }
  .code-preview pre {
    margin: 0;
    font-size: 11px;
    font-family: 'SF Mono', 'Fira Code', monospace;
    color: var(--text-secondary);
    white-space: pre;
  }
  textarea {
    width: 100%;
    box-sizing: border-box;
    background: var(--bg-input);
    color: var(--text-primary);
    border: none;
    padding: 12px;
    font-size: 13px;
    font-family: inherit;
    resize: vertical;
  }
  textarea:focus { outline: none; }
  textarea::placeholder { color: var(--text-muted); }
  .comment-actions {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 12px;
    border-top: 1px solid var(--border-light);
  }
  .hint {
    font-size: 11px;
    color: var(--text-muted);
  }
  .comment-actions button {
    background: var(--submit-bg);
    color: var(--text-white);
    border: none;
    border-radius: 4px;
    padding: 4px 16px;
    font-size: 13px;
    cursor: pointer;
  }
  .comment-actions button:hover { background: var(--submit-hover); }
  .comment-actions button:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
