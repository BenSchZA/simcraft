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
	<ellipse cx="50" cy="20" rx="40" ry="8" />
	<line x1="10" y1="20" x2="10" y2="80" />
	<line x1="90" y1="20" x2="90" y2="80" />
	<ellipse cx="50" cy="80" rx="40" ry="8" />
</svg>
