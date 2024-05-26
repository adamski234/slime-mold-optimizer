#!/bin/bash
functions="ackley,schwefel,brown,rastrigin,schwefel2,solomon"

runs_per_set=64

swarm_count=20
swarm_size=20
iterations=750

# swarm parameters
migration_thresholds=(0.01 0.03 0.06 0.09 0.12 0.15 0.18 0.2)

# slimes
z_parameters=(0.01 0.03 0.06 0.09 0.12 0.15 0.18 0.2)

# particles
social_coeff=0.6
cognitive_coeff=0.5
inertia_coeff=0.7

rm -rf output_slime
rm -rf output_particles
mkdir -p output_slime
mkdir -p output_particles

cargo build --release

for migration_threshold in "${migration_thresholds[@]}"
do
	# process slimes
	for z_param in "${z_parameters[@]}"
	do
		./target/release/slimes --functions=$functions --try-count $runs_per_set --migration-threshold $migration_threshold --swarm-count $swarm_count \
			--pop-size $swarm_size --iterations $iterations slime --z-parameter $z_param > "output_slime/z-param_"$z_param"_mig-thresh_"$migration_threshold".txt"
	done

	# process particles
	./target/release/slimes --functions=$functions --try-count $runs_per_set --migration-threshold $migration_threshold --swarm-count $swarm_count \
			--pop-size $swarm_size --iterations $iterations particles --social-coeff $social_coeff --cognitive-coeff $cognitive_coeff \
			--inertia-coeff $inertia_coeff > "output_particles/mig-thresh_"$migration_threshold".txt"
done