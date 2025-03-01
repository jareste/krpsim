# KRPSim

KRPSim is a Rust-based project designed to simulate resource management and optimization processes efficiently.

## Features
- **Resource Management**: Handles complex resource allocation.
- **Optimization Algorithms**: Implements efficient strategies for maximizing resources.
- **Command-line Interface**: Easy-to-use CLI for configuring simulations.

## Installation
Clone the repository and build with Cargo:
```bash
git clone https://github.com/jareste/krpsim.git
cd krpsim
cargo build --release
```

## Usage
Run simulations using:
```bash
./target/release/krpsim [options]
```

## Implemented Algorithms
1. **Dijkstra**: Finds shortest paths in graphs using a greedy approach.
2. **Ant Colony Optimization (ACO)**: A probabilistic technique for finding optimal paths.
3. **Tabu Search (Tabu)**: Metaheuristic search to avoid local optima.
4. **Genetic Algorithm (GA)**: Evolutionary technique for optimization using selection and mutation.
5. **Simulated Annealing (SA)**: Probabilistic method for approximating global optima.
6. **A\***: Pathfinding and graph traversal algorithm.
7. **IDA\***: Iterative deepening variant of A* for memory efficiency.
8. **Serial Generation Scheme (SGS)**: For scheduling and resource allocation.
