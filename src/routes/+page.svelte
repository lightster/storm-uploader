<script>
	import { invoke } from '@tauri-apps/api/core';
	import { listen } from '@tauri-apps/api/event';
	import { check } from '@tauri-apps/plugin-updater';
	import { relaunch } from '@tauri-apps/plugin-process';
	import { onMount } from 'svelte';
	import UploadList from '$lib/UploadList.svelte';

	let uploads = $state([]);
	let updateVersion = $state(null);
	let updateStatus = $state('idle');

	onMount(async () => {
		uploads = await invoke('get_uploads');

		const unlistenUploads = await listen('upload-changed', (event) => {
			uploads = event.payload;
		});

		const unlistenUpdate = await listen('update-available', (event) => {
			updateVersion = event.payload;
		});

		return () => {
			unlistenUploads();
			unlistenUpdate();
		};
	});

	async function installUpdate() {
		updateStatus = 'updating';
		try {
			const update = await check();
			if (update) {
				await update.downloadAndInstall();
				await relaunch();
			}
		} catch (e) {
			console.error('Update failed:', e);
			updateStatus = 'error';
		}
	}
</script>

<div class="flex flex-col h-screen">
	<!-- Header -->
	<div class="flex items-center justify-between px-3 py-2.5 border-b border-zinc-800">
		<h1 class="text-sm font-semibold text-zinc-200">Storm Uploader</h1>
	</div>

	<!-- Update banner -->
	{#if updateVersion}
		<div class="flex items-center justify-between px-3 py-1.5 bg-blue-900/50 border-b border-blue-800 text-xs">
			<span class="text-blue-200">v{updateVersion} available</span>
			{#if updateStatus === 'updating'}
				<span class="text-blue-300">Updating...</span>
			{:else if updateStatus === 'error'}
				<button
					onclick={installUpdate}
					class="text-blue-300 hover:text-blue-100 font-medium"
				>Retry</button>
			{:else}
				<button
					onclick={installUpdate}
					class="text-blue-300 hover:text-blue-100 font-medium"
				>Update</button>
			{/if}
		</div>
	{/if}

	<!-- Content -->
	<UploadList {uploads} />
</div>
