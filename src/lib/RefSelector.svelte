<script lang="ts">
  type RefInfo = {
    name: string;
    refType: 'branch' | 'tag' | 'worktree';
  };

  let { refs = [], selected = $bindable(''), label = '' }: {
    refs: RefInfo[];
    selected: string;
    label: string;
  } = $props();
</script>

<div class="ref-selector">
  <label>{label}</label>
  <select bind:value={selected}>
    <option value="" disabled>Select ref...</option>
    {#each refs as ref}
      <option value={ref.name}>{ref.name} ({ref.refType})</option>
    {/each}
  </select>
</div>

<style>
  .ref-selector {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  label {
    font-weight: 600;
    font-size: 13px;
    color: #aaa;
  }
  select {
    background: #2a2a2a;
    color: #e0e0e0;
    border: 1px solid #444;
    border-radius: 4px;
    padding: 4px 8px;
    font-size: 13px;
  }
</style>
