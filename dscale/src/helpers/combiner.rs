//! Value combination utilities for gathering multiple responses.
//!
//! This module provides the `Combiner` struct for collecting a fixed number
//! of values before processing them as a group. This is particularly useful
//! for implementing quorum-based algorithms, consensus protocols, and other
//! distributed system patterns that require waiting for multiple responses.

use std::usize;

/// A runtime-configured collector for gathering multiple values.
///
/// `Combiner` is designed for scenarios where you need to collect exactly a specified
/// number of values before proceeding with computation. It's particularly useful for
/// implementing distributed system patterns such as:
///
/// - **Quorum Systems**: Wait for a majority of responses before making decisions
/// - **Consensus Protocols**: Collect votes or acknowledgments from multiple processes
/// - **Redundant Requests**: Gather responses from multiple replicas
/// - **Batch Processing**: Accumulate items until a threshold is reached
///
///
/// # Generic Parameters
///
/// - `T`: The type of values to collect. Must implement `Sized`.
///
/// #[derive(Clone)]
/// struct VoteMessage {
///     proposal_id: u64,
///     vote: bool,
/// }
/// impl Message for VoteMessage {}
///
/// struct ConsensusProcess {
///     proposal_id: u64,
///     vote_collector: Option<Combiner<bool, 3>>,
/// }
///
/// impl ProcessHandle for ConsensusProcess {
///     fn start(&mut self) {
///         // Start a new consensus round
///         self.proposal_id = 1;
///         self.vote_collector = Some(Combiner::new(3));
///
///         // Send vote requests to other processes
///         // send_to(1, VoteMessage { proposal_id: 1, vote: true });
///         // send_to(2, VoteMessage { proposal_id: 1, vote: true });
///         // send_to(3, VoteMessage { proposal_id: 1, vote: false });
///     }
///
///     fn on_message(&mut self, from: ProcessId, message: MessagePtr) {
///         if let Some(vote_msg) = message.try_as::<VoteMessage>() {
///             if vote_msg.proposal_id == self.proposal_id {
///                 if let Some(ref mut collector) = self.vote_collector {
///                     if let Some(votes) = collector.combine(vote_msg.vote) {
///                         debug_process!("Collected all votes: {:?}", votes);
///                         let yes_count = votes.iter().filter(|&&v| v).count();
///                         let consensus = yes_count >= 2; // Majority rule
///                         debug_process!("Consensus result: {}", consensus);
///                         self.vote_collector = None; // Reset for next round
///                     }
///                 }
///             }
///         }
///     }
///
///     fn on_timer(&mut self, id: TimerId) {}
/// }
/// # impl Default for ConsensusProcess {
/// #     fn default() -> Self {
/// #         Self { proposal_id: 0, vote_collector: None }
/// #     }
/// # }
/// ```
///
/// ## Response Aggregation
///
/// ```rust
/// use dscale::helpers::Combiner;
///
/// #[derive(Debug)]
/// struct ServerResponse {
///     server_id: u32,
///     latency: u64,
///     data: String,
/// }
///
/// fn collect_responses() {
///     let mut collector: Combiner<ServerResponse> = Combiner::new(5);
///
///     // Simulate receiving responses from 5 servers
///     let responses = vec![
///         ServerResponse { server_id: 1, latency: 10, data: "result1".to_string() },
///         ServerResponse { server_id: 2, latency: 15, data: "result2".to_string() },
///         ServerResponse { server_id: 3, latency: 8,  data: "result3".to_string() },
///         ServerResponse { server_id: 4, latency: 12, data: "result4".to_string() },
///         ServerResponse { server_id: 5, latency: 20, data: "result5".to_string() },
///     ];
///
///     for response in responses {
///         if let Some(all_responses) = collector.combine(response) {
///             // All responses collected - find fastest
///             let fastest = all_responses.iter()
///                 .min_by_key(|r| r.latency)
///                 .unwrap();
///             println!("Fastest response from server {}: {}", fastest.server_id, fastest.data);
///             break;
///         }
///     }
/// }
/// ```
///
///
/// # Common Use Cases in Distributed Systems
///
/// - **Byzantine Fault Tolerance**: Collect 2f+1 responses in f-fault-tolerant systems
/// - **Read Quorums**: Wait for majority of replicas before returning data
/// - **Write Acknowledgments**: Ensure sufficient replicas confirm writes
/// - **Leader Election**: Collect votes from majority of processes
/// - **Consensus Algorithms**: Gather proposals or votes for Raft, PBFT, etc.
///
pub struct Combiner<T: Sized> {
    values: Vec<T>,
    threshold: usize,
    idx: usize,
}

impl<T: Sized> Combiner<T> {
    /// Creates a new combiner that will collect exactly `threshold` values.
    ///
    /// This constructor initializes an empty combiner ready to accept values
    /// through the [`combine`] method. The combiner will return `None` from
    /// [`combine`] until exactly `threshold` values have been provided.
    ///
    /// # Requirements
    ///
    /// The `threshold` must be greater than 0. This is enforced by a debug
    /// assertion to catch programming errors during development.
    ///
    /// # Panics
    ///
    /// In debug builds, panics if `threshold` is 0.
    ///
    /// [`combine`]: Combiner::combine
    pub fn new(threshold: usize) -> Self {
        debug_assert!(
            threshold > 0,
            "Combinter threshold should be greater than zero"
        );
        Self {
            values: Vec::with_capacity(threshold),
            threshold,
            idx: 0,
        }
    }

    /// Adds a value to the combiner and returns the complete collection when ready.
    ///
    /// This method accepts one value and adds it to the internal collection.
    /// It returns:
    /// - `None` if fewer than `threshold` values have been collected
    /// - `Some(&[T])` when exactly `threshold` values have been collected
    ///
    /// Once a complete collection is returned, the combiner is considered
    /// "consumed" and subsequent calls will always return `None`.
    ///
    /// # Behavior
    ///
    /// - **Before Completion**: Returns `None` and stores the value internally
    /// - **At Completion**: Returns `Some(slice)` containing all values in order
    /// - **After Completion**: Always returns `None` (combiner is exhausted)
    ///
    /// # Parameters
    ///
    /// * `value` - A value of type `T` to add to the collection
    ///
    /// # Returns
    ///
    /// - `None` if the collection is not yet complete
    /// - `Some(&[T])` when the collection is complete, containing all values in insertion order
    ///
    /// #[derive(Debug)]
    /// enum Response {
    ///     Success(String),
    ///     Error(u32),
    /// }
    ///
    /// fn handle_responses() {
    ///     let mut collector: Combiner<Response> = Combiner::new(3);
    ///
    ///     // Process responses as they arrive
    ///     let responses = [
    ///         Response::Success("OK".to_string()),
    ///         Response::Error(404),
    ///         Response::Success("Done".to_string()),
    ///     ];
    ///
    ///     for response in responses {
    ///         if let Some(all_responses) = collector.combine(response) {
    ///             let errors: Vec<_> = all_responses.iter()
    ///                 .filter_map(|r| match r {
    ///                     Response::Error(code) => Some(code),
    ///                     _ => None,
    ///                 })
    ///                 .collect();
    ///
    ///             if !errors.is_empty() {
    ///                 println!("Received errors: {:?}", errors);
    ///             } else {
    ///                 println!("All responses successful");
    ///             }
    ///             break;
    ///         }
    ///     }
    /// }
    /// ```
    ///
    pub fn combine(&mut self, value: T) -> Option<&[T]> {
        if self.idx >= self.threshold {
            return None;
        }

        self.values.push(value);
        self.idx += 1;

        if self.idx == self.threshold {
            Some(&self.values)
        } else {
            None
        }
    }
}
