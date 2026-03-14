<script>
	import { invoke } from '@tauri-apps/api/core';
	import { enable, disable, isEnabled } from '@tauri-apps/plugin-autostart';

	let { config, onSave } = $props();

	let watchDir = $state(config.watchDir);
	let apiUrl = $state(config.apiUrl);
	let autostart = $state(config.autostart);
	let saved = $state(false);

	$effect(() => {
		isEnabled().then((enabled) => {
			autostart = enabled;
		});
	});

	async function handleSave() {
		if (autostart) {
			await enable();
		} else {
			await disable();
		}

		const newConfig = { watchDir, apiUrl, autostart };
		await invoke('save_config_cmd', { config: newConfig });
		onSave?.(newConfig);

		saved = true;
		setTimeout(() => { saved = false; }, 2000);
	}
</script>

<div class="flex-1 overflow-y-auto px-3 pb-3">
	<div class="space-y-4">
		<div>
			<label for="watch-dir" class="block text-[11px] uppercase tracking-wider text-zinc-500 mb-1.5">
				Watch folder
			</label>
			<input
				id="watch-dir"
				type="text"
				bind:value={watchDir}
				class="w-full rounded-md border border-zinc-700 bg-zinc-800 px-2.5 py-1.5 text-sm text-zinc-200 outline-none focus:border-blue-500 transition-colors"
			/>
		</div>

		<div>
			<label for="api-url" class="block text-[11px] uppercase tracking-wider text-zinc-500 mb-1.5">
				API URL
			</label>
			<input
				id="api-url"
				type="text"
				bind:value={apiUrl}
				class="w-full rounded-md border border-zinc-700 bg-zinc-800 px-2.5 py-1.5 text-sm text-zinc-200 outline-none focus:border-blue-500 transition-colors"
			/>
		</div>

		<div class="flex items-center justify-between">
			<label for="autostart" class="text-sm text-zinc-300">Launch at login</label>
			<button
				id="autostart"
				role="switch"
				aria-checked={autostart}
				onclick={() => { autostart = !autostart; }}
				class="relative inline-flex h-5 w-9 items-center rounded-full transition-colors {autostart ? 'bg-blue-500' : 'bg-zinc-600'}"
			>
				<span class="inline-block h-3.5 w-3.5 rounded-full bg-white transition-transform {autostart ? 'translate-x-4' : 'translate-x-0.5'}" />
			</button>
		</div>

		<button
			onclick={handleSave}
			class="w-full rounded-md bg-blue-600 px-3 py-1.5 text-sm font-medium text-white hover:bg-blue-500 transition-colors"
		>
			{saved ? 'Saved!' : 'Save'}
		</button>
	</div>
</div>
