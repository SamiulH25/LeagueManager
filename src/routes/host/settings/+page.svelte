<script lang="ts">
  import { goto } from "$app/navigation";
  import RaceControlShell from "$lib/components/layout/RaceControlShell.svelte";
  import TimingPanel from "$lib/components/racing/TimingPanel.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Input from "$lib/components/ui/Input.svelte";
  import { api } from "$lib/api";
  import type { HostSettings } from "$lib/types";
  import { FolderSearch, Save } from "@lucide/svelte";
  import { getContext, onMount } from "svelte";
  import type { AppStore } from "$lib/stores/app.svelte";

  const store = getContext<AppStore>("app");
  let settings = $state<HostSettings>({
    assettoServerPath: "",
    acInstallPath: "",
    httpPort: 8081,
    gamePort: 9600,
    adminPassword: "leaguemgr",
    publicIpOverride: "",
  });
  let httpPort = $state("8081");
  let gamePort = $state("9600");
  let saving = $state(false);
  let detecting = $state(false);
  let message = $state<string | null>(null);
  let error = $state<string | null>(null);

  onMount(async () => {
    settings = await api.getHostSettings();
    httpPort = String(settings.httpPort);
    gamePort = String(settings.gamePort);
  });

  async function detectPaths() {
    detecting = true;
    error = null;
    try {
      const suggestions = await api.detectPaths();
      if (suggestions.assettoServerPath) {
        settings.assettoServerPath = suggestions.assettoServerPath;
      }
      if (suggestions.acInstallPath) {
        settings.acInstallPath = suggestions.acInstallPath;
      }
      message = "Path suggestions applied where found on disk.";
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      detecting = false;
    }
  }

  async function save() {
    saving = true;
    error = null;
    message = null;
    try {
      await api.saveHostSettings({
        ...settings,
        httpPort: Number(httpPort) || 8081,
        gamePort: Number(gamePort) || 9600,
      });
      message = "Pit config saved.";
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      saving = false;
    }
  }

  async function logout() {
    await store.logout();
    goto("/login");
  }
</script>

{#snippet header()}
  <div>
    <p class="font-label text-[0.65rem] text-[var(--color-red)]">Pit config</p>
    <h1 class="font-display text-3xl leading-none text-white lg:text-4xl">SERVER SETUP</h1>
  </div>
{/snippet}

{#if store.state?.session}
  <RaceControlShell
    mode="host"
    session={store.state.session}
    activeHref="/host/settings"
    {header}
    onLogout={logout}
  >
    <div class="mx-auto max-w-2xl">
      <TimingPanel title="AssettoServer paths" subtitle="Required before launching races">
        <div class="space-y-4">
          <div>
            <p class="mb-1 font-label text-[0.6rem] text-[var(--color-dim)]">AssettoServer folder</p>
            <Input
              bind:value={settings.assettoServerPath}
              placeholder="C:\Users\You\AssettoServer"
              mono
            />
            <p class="mt-1 text-xs text-[var(--color-muted)]">
              Folder containing AssettoServer.exe (Windows) or AssettoServer binary.
            </p>
          </div>

          <div>
            <p class="mb-1 font-label text-[0.6rem] text-[var(--color-dim)]">AC install path</p>
            <Input
              bind:value={settings.acInstallPath}
              placeholder="C:\Program Files (x86)\Steam\steamapps\common\assettocorsa"
              mono
            />
            <p class="mt-1 text-xs text-[var(--color-muted)]">
              Used later for content validation and results paths.
            </p>
          </div>

          <Button variant="secondary" size="sm" loading={detecting} onclick={detectPaths}>
            <FolderSearch class="size-3.5" strokeWidth={2} />
            Auto-detect paths
          </Button>
        </div>
      </TimingPanel>

      <div class="mt-4">
      <TimingPanel title="Network" subtitle="Ports and public address">
        <div class="grid gap-4 sm:grid-cols-2">
          <div>
            <p class="mb-1 font-label text-[0.6rem] text-[var(--color-dim)]">HTTP port</p>
            <Input bind:value={httpPort} type="number" mono />
          </div>
          <div>
            <p class="mb-1 font-label text-[0.6rem] text-[var(--color-dim)]">Game port (UDP/TCP)</p>
            <Input bind:value={gamePort} type="number" mono />
          </div>
          <div class="sm:col-span-2">
            <p class="mb-1 font-label text-[0.6rem] text-[var(--color-dim)]">Public IP override</p>
            <Input bind:value={settings.publicIpOverride} placeholder="Leave blank for auto-detect" mono />
          </div>
          <div class="sm:col-span-2">
            <p class="mb-1 font-label text-[0.6rem] text-[var(--color-dim)]">Admin password</p>
            <Input bind:value={settings.adminPassword} type="password" mono />
          </div>
        </div>
      </TimingPanel>
      </div>

      {#if message}
        <p class="mt-4 text-sm text-[var(--color-green)]">{message}</p>
      {/if}
      {#if error}
        <p class="mt-4 text-sm text-[var(--color-red)]">{error}</p>
      {/if}

      <Button variant="primary" class="mt-6 w-full sm:w-auto" loading={saving} onclick={save}>
        <Save class="size-3.5" strokeWidth={2.5} />
        Save pit config
      </Button>
    </div>
  </RaceControlShell>
{/if}
