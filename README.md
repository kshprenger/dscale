# DScale

A fast, deterministic simulation framework for testing and benchmarking distributed systems. It simulates network latency, bandwidth constraints, and process execution in an event-driven environment with support for both single-threaded and parallel execution modes.

## Usage

### 1. Install

In your project

```shell
cargo add dscale
```

### 2. Define Messages

Messages must implement the `Message` trait, which allows defining a `virtual_size` for bandwidth simulation.

```rust
use dscale::Message;

struct MyMessage {
    data: u32,
}

impl Message for MyMessage {
    fn virtual_size(&self) -> usize {
        // Size in bytes used for bandwidth simulation.
        // Can be much bigger than real memory size to simulate heavy payloads.
        1000
    }
}

// Or (if there is no need in bandwidth)
impl Message for MyMessage {}
```

### 3. Implement Process Logic

Implement `ProcessHandle` to define how your process reacts to initialization, messages, and timers.

```rust
use dscale::{ProcessHandle, Rank, MessagePtr, TimerId, Jiffies};
use dscale::{broadcast, send_to, schedule_timer_after, rank, debug_process};
use dscale::global::configuration;

#[derive(Default)]
struct MyProcess;

impl ProcessHandle for MyProcess {
    fn on_start(&mut self) {
        debug_process!("Starting process {} of {}", rank(), configuration::process_number());
        schedule_timer_after(Jiffies(100));
    }

    fn on_message(&mut self, from: Rank, message: MessagePtr) {
        if let Some(msg) = message.try_as_type::<MyMessage>() {
            debug_process!("Received message from {}: {}", from, msg.data);
        }
    }

    fn on_timer(&mut self, _id: TimerId) {
        broadcast(MyMessage { data: 42 });
    }
}
```

### 4. Run the Simulation

Use `SimulationBuilder` to configure the topology, network constraints, and start the simulation.

```rust
use dscale::{SimulationBuilder, Jiffies, BandwidthConfig, Distributions};

fn main() {
    let mut runner = SimulationBuilder::default()
        .add_pool::<MyClient>("Client", 1)
        .add_pool::<MyServer>("Server", 3)
        .within_pool_latency("Client", Distributions::Uniform(Jiffies(1), Jiffies(5)))
        .within_pool_latency("Server", Distributions::Uniform(Jiffies(1), Jiffies(5)))
        .between_pool_latency("Client", "Server", Distributions::Normal {
            mean: Jiffies(10),
            std_dev: Jiffies(2),
            low: Jiffies(5),
            high: Jiffies(20),
        })
        .vnic_bandwidth(BandwidthConfig::Bounded(1000)) // 1000 bytes per Jiffy
        .time_budget(Jiffies(1_000_000))
        .simple()
        .build();

    runner.run_full_budget();
}
```

#### Parallel Execution

For large simulations, enable parallel execution to distribute process steps across multiple threads:

```rust
let mut runner = SimulationBuilder::default()
    .add_pool::<MyProcess>("Nodes", 1000)
    .within_pool_latency("Nodes", Distributions::Uniform(Jiffies(1), Jiffies(10)))
    .time_budget(Jiffies(1_000_000))
    .parallel(Threads::Specific(8)) // use 8 worker threads
    .build();

runner.run_full_budget();
```

When is parallel mode efficient?

1. A lot of simulated processes (at least 200-300)
2. on_message execution takes most of simulation time
3. Independent work inside on_message (not so much synchronization)

## Public API

### Simulation Control

- **`SimulationBuilder`**: Configures the simulation environment.
  - `default`: Creates simulation with no processes and default parameters.
  - `seed`: Sets the random seed for deterministic execution.
  - `time_budget`: Sets the maximum simulation duration.
  - `add_pool`: Creates a named pool of processes. (All processes also join `GLOBAL_POOL`)
  - `within_pool_latency(pool, distribution)`: Configures latency between processes within a pool.
  - `between_pool_latency(pool_a, pool_b, distribution)`: Configures latency between two pools (symmetric). Every pool pair must have latency configured before calling `build`.
  - `vnic_bandwidth`: Configures per-process network bandwidth limits for "virtual" NIC.
    - `Bounded(usize)`: Limits bandwidth (bytes per jiffy).
    - `Unbounded`: No bandwidth limits (default).
  - `simple`: Selects single-threaded execution (default).
  - `parallel(threads)`: Selects parallel execution with the given number of worker threads.
  - `build`: Finalizes configuration and returns a simulation runner.
- **`run_full_budget`**: Runs the simulation until the time budget is exhausted.
- **`run_steps`**: Runs the simulation until it performs the requested number of steps or the global budget is exhausted.
- **`run_sub_budget`**: Runs the simulation until the sub-budget starting from current timepoint or global budget are exhausted.

### Network Topology

- **`GLOBAL_POOL`**:
  - Implicit pool containing all processes. `broadcast` uses this pool.
- **`Distributions`**:
  - `Uniform(low, high)`: Uniform distribution over `[low, high]`.
  - `Bernoulli(p, value)`: With probability `p` the latency is `value`, otherwise 0.
  - `Normal { mean, std_dev, low, high }`: Truncated normal distribution clamped to `[low, high]`.

### Process Interaction (Context-Aware)

These functions are available globally but must be called within the context of a running process step.

- **`broadcast`**: Shortcut for `broadcast_within_pool(GLOBAL_POOL)`.
- **`broadcast_within_pool`**: Sends a message to all processes within a named pool.
- **`send_to`**: Sends a message to a specific process by rank.
- **`send_random`**: Shortcut for `send_random_from_pool(GLOBAL_POOL)`.
- **`send_random_from_pool`**: Sends a message to a random process within a named pool.
- **`schedule_timer_after`**: Schedules a timer for the current process, returns a `TimerId`.
- **`rank`**: Returns the rank of the currently executing process. (Ranks start at 0)
- **`now`**: Returns the current simulation time.
- **`list_pool`**: Returns a slice of all process ranks in a pool.
- **`choose_from_pool`**: Picks a random process rank from a named pool.
- **`global_unique_id`**: Generates a globally unique monotonic ID.

### Configuration (`dscale::global::configuration`)

- **`seed`**: Returns the deterministic seed for the current process.
- **`process_number`**: Returns total number of processes in the simulation.

### Key-Value Store (`dscale::global::kv`)

Thread-safe store for passing shared state, metrics, or configuration between processes or back to the host.

- **`set(key, value)`**: Stores a value under the given key.
- **`get(key) -> T`**: Retrieves a clone of the value (panics if missing or wrong type).
- **`modify(key, f)`**: Mutates the value in place.

### Helpers (`dscale::helpers`)

- **`debug_process!`**: A macro that logs with current simulation time and process rank prepended.
- **`Combiner`**: Collects values until a threshold is reached, then yields them all at once. Useful for quorum-based logic.

### Message Downcasting (`MessagePtr`)

- **`try_as_type::<T>()`**: Attempts to downcast to `T`, returns `Option<&T>`.
- **`as_type::<T>()`**: Downcasts to `T`, panics if the type does not match.
- **`is::<T>()`**: Returns `true` if the message is of type `T`.

## Logging Configuration (`RUST_LOG`)

DScale output is controlled via the `RUST_LOG` environment variable.

- **`RUST_LOG=info`**: Shows high-level simulation status and a progress bar.
- **`RUST_LOG=debug`**: Enables all `debug_process!` macro output and all internal simulation events.
- **`RUST_LOG=full::path::to::your::file::or::crate=debug,another::path=debug`**: Filter events only for your specific file or crate.

Note: `RUST_LOG=debug` and path-level debug filters only work without the `--release` flag.

## Thanks to

- https://gitlab.com/whirl-framework
- https://github.com/jepsen-io/maelstrom
- https://github.com/systems-group/anysystem
- https://www.open-mpi.org
