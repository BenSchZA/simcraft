<script lang="ts">
	export let fill = 'none';
	export let stroke = 'currentColor';
	export let strokeWidth = '4';
	export let classes = '';
</script>

<svg
	width="100%"
	height="100%"
	viewBox="0 0 100 100"
	class={classes}
	xmlns="http://www.w3.org/2000/svg"
	{fill}
	{stroke}
	stroke-width={strokeWidth}
>
	<polygon points="50,90 10,10 90,10" />
</svg>
