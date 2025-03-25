<script lang="ts">
	// import * as wasm from 'simcraft';
	import * as simcraft_wasm from '../../../../crates/simcraft_web/Cargo.toml';

	async function runSimulation() {
		const simcraft = await simcraft_wasm();

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

		let simulation = simcraft.WebSimulation.new(
			JSON.stringify(processes),
			JSON.stringify(connections)
		);
		let results = simulation.step_n(100);
		console.log(results);
	}

	runSimulation();
</script>
