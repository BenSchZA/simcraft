<script lang="ts">
	export let fill = 'none';
	export let stroke = 'currentColor';
	export let strokeWidth = '4';
	export let classes = '';
</script>

<svg
	viewBox="0 0 100 100"
	class={classes}
	xmlns="http://www.w3.org/2000/svg"
	{fill}
	{stroke}
	stroke-width={strokeWidth}
>
	<circle cx="50" cy="50" r="40" />
</svg>
