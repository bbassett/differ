<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount, onDestroy } from 'svelte';

  let pending = $state(0);
  let interval: ReturnType<typeof setInterval>;

  async function poll() {
    try {
      pending = await invoke<number>('get_queue_length');
    } catch {
      // ignore polling errors
    }
  }

  onMount(() => {
    poll();
    interval = setInterval(poll, 2000);
  });

  onDestroy(() => {
    clearInterval(interval);
  });
</script>

<div class="queue-status" class:has-items={pending > 0}>
  {#if pending > 0}
    {pending} comment{pending !== 1 ? 's' : ''} pending
  {:else}
    Queue empty
  {/if}
</div>

<style>
  .queue-status {
    font-size: 12px;
    color: #666;
    padding: 4px 8px;
    border-radius: 4px;
    background: #2a2a2a;
  }
  .queue-status.has-items {
    color: #d29922;
    background: #d2992215;
  }
</style>
