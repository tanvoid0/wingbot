<script lang="ts">
  import LeagueLauncher from '$lib/components/LeagueLauncher.svelte';
</script>

<svelte:head>
  <title>League Launcher - Wingbot</title>
</svelte:head>

<div class="page-container">
  <LeagueLauncher />
</div>

<style>
  .page-container {
    min-height: 100vh;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    padding: 20px;
  }
</style>
