<script lang="ts">
	import { WebSimulation } from 'simcraft_web';

	async function runSimulation() {
		let processes = [
			{
				type: 'Stepper',
				id: 'stepper'
			},
			{
				type: 'Source',
				id: 'source-1'
			},
			{
				type: 'Pool',
				id: 'pool-1'
			}
		];

		let connections = [
			{
				id: 'connection-1',
				sourceID: 'source-1',
				sourcePort: 'out',
				targetID: 'pool-1',
				targetPort: 'in'
			}
		];

		let simulation = WebSimulation.new(JSON.stringify(processes), JSON.stringify(connections));
		let results = simulation.step_n(100);
		console.log(results);
	}

	runSimulation();
</script>
