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
    background: #2a2a2a;
    border: 1px solid #555;
    border-radius: 8px;
    box-shadow: 0 4px 24px rgba(0, 0, 0, 0.5);
    z-index: 100;
  }
  .comment-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 12px;
    border-bottom: 1px solid #444;
  }
  .comment-location {
    font-family: monospace;
    font-size: 12px;
    color: #79b8ff;
  }
  .close-btn {
    background: none;
    border: none;
    color: #888;
    cursor: pointer;
    font-size: 14px;
    padding: 0 4px;
  }
  .close-btn:hover { color: #fff; }
  .code-preview {
    max-height: 80px;
    overflow: auto;
    padding: 8px 12px;
    background: #1e1e1e;
    border-bottom: 1px solid #444;
  }
  .code-preview pre {
    margin: 0;
    font-size: 11px;
    font-family: 'SF Mono', 'Fira Code', monospace;
    color: #aaa;
    white-space: pre;
  }
  textarea {
    width: 100%;
    box-sizing: border-box;
    background: #1e1e1e;
    color: #e0e0e0;
    border: none;
    padding: 12px;
    font-size: 13px;
    font-family: inherit;
    resize: vertical;
  }
  textarea:focus { outline: none; }
  textarea::placeholder { color: #666; }
  .comment-actions {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 12px;
    border-top: 1px solid #444;
  }
  .hint {
    font-size: 11px;
    color: #666;
  }
  .comment-actions button {
    background: #347d39;
    color: #fff;
    border: none;
    border-radius: 4px;
    padding: 4px 16px;
    font-size: 13px;
    cursor: pointer;
  }
  .comment-actions button:hover { background: #3e8e41; }
  .comment-actions button:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
