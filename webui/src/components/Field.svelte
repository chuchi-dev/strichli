<script lang="ts">
	type Props = {
		label: string;
		value: string;
		type?: 'text' | 'email' | 'password';
		placeholder?: string;
		autocomplete?: AutoFill;
		required?: boolean;
		hint?: string;
	};

	let {
		label,
		value = $bindable(),
		type = 'text',
		placeholder,
		autocomplete,
		required = false,
		hint,
	}: Props = $props();
</script>

<label class="field">
	<span class="label">{label}</span>
	<input
		{type}
		bind:value
		{placeholder}
		{autocomplete}
		{required}
		spellcheck="false"
	/>
	{#if hint}
		<span class="hint">{hint}</span>
	{/if}
</label>

<style lang="scss">
	@use 'styles/mixins' as *;

	.field {
		display: grid;
		gap: 0.3rem;
	}

	.label {
		@include caps;
		color: var(--ink-2);
	}
</style>
