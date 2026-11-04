<script lang="ts">
	import type { EventRow, EventTable, Range } from '@/api/sites';
	import { getEvents } from '@/api/sites';
	import { num } from '@/lib/format';

	type Props = {
		siteId: string;
		range: Range;
		table: EventTable;
		/** the data fields shown as columns */
		columns: string[];
	};

	const { siteId, range, table: loaded, columns: initial }: Props = $props();

	const MAX_COLUMNS = 5;

	// svelte-ignore state_referenced_locally
	let table = $state(loaded);
	// svelte-ignore state_referenced_locally
	let columns = $state(initial);
	let filter = $state('');
	let error = $state('');

	const value = (row: EventRow, field: string) => {
		const v = row.values[field];
		return v === null || v === undefined ? '' : String(v);
	};

	const rows = $derived.by(() => {
		const f = filter.trim().toLowerCase();
		if (!f) return table.rows;
		return table.rows.filter(
			r =>
				r.key.toLowerCase().includes(f) ||
				columns.some(c => value(r, c).toLowerCase().includes(f)),
		);
	});
	const total = $derived(rows.reduce((a, r) => a + r.count, 0));
	const addable = $derived(table.fields.filter(f => !columns.includes(f)));

	async function setColumns(next: string[]) {
		error = '';
		try {
			table = await getEvents(siteId, range, next);
			columns = next;
		} catch (e: any) {
			error = e.message;
		}
	}

	function add(e: Event) {
		const select = e.currentTarget as HTMLSelectElement;
		if (select.value) setColumns([...columns, select.value]);
		select.value = '';
	}

	function download() {
		const esc = (s: string) => `"${s.replaceAll('"', '""')}"`;
		const lines = [
			['key', ...columns, 'count'].map(esc).join(','),
			...rows.map(r =>
				[r.key, ...columns.map(c => value(r, c)), String(r.count)]
					.map(esc)
					.join(','),
			),
		];
		const blob = new Blob([lines.join('\n') + '\n'], { type: 'text/csv' });
		const a = document.createElement('a');
		a.href = URL.createObjectURL(blob);
		a.download = `events-${range.from}-${range.to}.csv`;
		a.click();
		URL.revokeObjectURL(a.href);
	}
</script>

<div class="events">
	<div class="tools">
		<input
			type="search"
			placeholder="Filter events"
			aria-label="Filter events"
			bind:value={filter}
			spellcheck="false"
		/>

		<div class="columns">
			{#each columns as c (c)}
				<button
					class="chip"
					title="Remove the column {c}"
					onclick={() => setColumns(columns.filter(x => x !== c))}
				>
					{c}
					<span aria-hidden="true">×</span>
				</button>
			{/each}
			{#if addable.length && columns.length < MAX_COLUMNS}
				<select onchange={add} aria-label="Add a column">
					<option value="">Add a column</option>
					{#each addable as f (f)}
						<option value={f}>{f}</option>
					{/each}
				</select>
			{/if}
		</div>

		<button class="download" onclick={download} disabled={!rows.length}>
			Download CSV
		</button>
	</div>

	{#if error}
		<p class="error">{error}</p>
	{/if}

	<p class="summary">
		{#if rows.length}
			Found <strong>{num(rows.length)}</strong>
			{rows.length === 1 ? 'row' : 'rows'} with a total of
			<strong>{num(total)}</strong>
			events.
		{:else}
			No events in this time{filter ? ' that match the filter' : ''}. Send
			them with
			<code>strichli('signup', &#123; plan: 'pro' &#125;)</code>
			.
		{/if}
	</p>

	{#if rows.length}
		<div class="card">
			<table class="list">
				<thead>
					<tr>
						<th>Key</th>
						{#each columns as c (c)}
							<th>{c}</th>
						{/each}
						<th class="right">Count</th>
					</tr>
				</thead>
				<tbody>
					{#each rows as row, i (i)}
						<tr>
							<td class="key">{row.key}</td>
							{#each columns as c (c)}
								<td class:muted={!value(row, c)}>
									{value(row, c) || '–'}
								</td>
							{/each}
							<td class="right num">{num(row.count)}</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	{/if}
</div>

<style lang="scss">
	.tools {
		display: flex;
		flex-wrap: wrap;
		gap: 0.75rem;
		align-items: center;

		input[type='search'] {
			width: 16rem;
		}
	}

	.columns {
		display: flex;
		flex: 1;
		flex-wrap: wrap;
		gap: 0.4rem;
		align-items: center;

		select {
			padding: 0.3rem 0.4rem;
			font-size: 0.85rem;
		}
	}

	.chip {
		padding: 0.25rem 0.55rem;
		font-family: var(--mono);
		font-size: 0.8rem;
		background-color: var(--surface);
		border: 2px solid var(--border);
		border-radius: 999px;
		cursor: pointer;

		&:hover {
			background-color: var(--wash);
		}

		span {
			color: var(--muted);
		}
	}

	.download {
		padding: 0.4rem 0.8rem;
		font-family: var(--mono);
		font-size: 0.78rem;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		background-color: var(--surface);
		border: 2px solid var(--border);
		border-radius: var(--radius);
		box-shadow: 3px 3px 0 var(--shadow);
		cursor: pointer;

		&:active:not(:disabled) {
			transform: translate(3px, 3px);
			box-shadow: none;
		}

		&:disabled {
			opacity: 0.5;
			cursor: default;
		}
	}

	.summary {
		margin: 1rem 0;
		color: var(--ink-2);
	}

	.key {
		font-family: var(--mono);
		font-size: 0.9em;
	}

	code {
		font-size: 0.85em;
	}

	tr:last-child td {
		border-bottom: none;
	}
</style>
