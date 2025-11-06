<script lang="ts">
	import { goto } from '$app/navigation';
	import { notifications } from '$lib/stores/notifications';
	import { mutation } from '@urql/svelte';
	import { UPLOAD_TORRENT_MUTATION } from '$lib/graphql/mutations';

	let name = '';
	let description = '';
	let category = 'Movies';
	let tags = '';
	let torrentFile: FileList;
	let uploading = false;

	const categories = ['Movies', 'TV', 'Music', 'Games', 'Software', 'Books', 'Other'];

	const uploadMutation = mutation({ query: UPLOAD_TORRENT_MUTATION });

	async function handleSubmit() {
		if (!name || !category || !torrentFile || torrentFile.length === 0) {
			notifications.error('Please fill in all required fields');
			return;
		}

		uploading = true;

		try {
			// In a real implementation, you would upload the file first
			// and get back the torrent metadata
			const result = await uploadMutation({
				input: {
					name,
					description,
					category,
					tags: tags.split(',').map(t => t.trim()).filter(Boolean),
					infoHash: 'placeholder', // This would come from parsing the torrent file
					size: torrentFile[0].size
				}
			});

			if (result.error) {
				notifications.error(result.error.message);
			} else if (result.data?.uploadTorrent) {
				notifications.success('Torrent uploaded successfully!');
				goto(`/torrent/${result.data.uploadTorrent.id}`);
			}
		} catch (error) {
			notifications.error('An error occurred during upload');
		} finally {
			uploading = false;
		}
	}
</script>

<svelte:head>
	<title>Upload Torrent - Tracker Platform</title>
</svelte:head>

<div class="max-w-3xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
	<div class="card p-8">
		<h1 class="text-3xl font-bold text-primary mb-8">contributing to the void</h1>

		<form on:submit|preventDefault={handleSubmit} class="space-y-6">
			<!-- Torrent File -->
			<div>
				<label for="torrentFile" class="block text-sm font-medium text-primary mb-2">
					Torrent File *
				</label>
				<input
					id="torrentFile"
					type="file"
					accept=".torrent"
					bind:files={torrentFile}
					class="input"
					required
					disabled={uploading}
				/>
				<p class="mt-1 text-xs text-muted">Only .torrent files are accepted</p>
			</div>

			<!-- Name -->
			<div>
				<label for="name" class="block text-sm font-medium text-primary mb-2">
					Name *
				</label>
				<input
					id="name"
					type="text"
					bind:value={name}
					class="input"
					placeholder="Enter torrent name"
					required
					disabled={uploading}
				/>
			</div>

			<!-- Description -->
			<div>
				<label for="description" class="block text-sm font-medium text-primary mb-2">
					Description
				</label>
				<textarea
					id="description"
					bind:value={description}
					class="input min-h-[150px] resize-none"
					placeholder="Describe this torrent..."
					disabled={uploading}
				></textarea>
			</div>

			<!-- Category -->
			<div>
				<label for="category" class="block text-sm font-medium text-primary mb-2">
					Category *
				</label>
				<select
					id="category"
					bind:value={category}
					class="input"
					required
					disabled={uploading}
				>
					{#each categories as cat}
						<option value={cat}>{cat}</option>
					{/each}
				</select>
			</div>

			<!-- Tags -->
			<div>
				<label for="tags" class="block text-sm font-medium text-primary mb-2">
					Tags
				</label>
				<input
					id="tags"
					type="text"
					bind:value={tags}
					class="input"
					placeholder="tag1, tag2, tag3"
					disabled={uploading}
				/>
				<p class="mt-1 text-xs text-muted">Separate tags with commas</p>
			</div>

			<!-- Submit -->
			<div class="flex gap-4">
				<button
					type="submit"
					class="btn btn-primary flex-1"
					disabled={uploading}
				>
					{uploading ? 'sealing my fate...' : 'contributing to the void'}
				</button>
				<a href="/torrents" class="btn btn-secondary">
					giving up (as usual)
				</a>
			</div>
		</form>
	</div>
</div>
