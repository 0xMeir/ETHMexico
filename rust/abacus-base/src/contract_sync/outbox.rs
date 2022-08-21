use std::cmp::min;
use std::time::Duration;

use tokio::time::sleep;
use tracing::{debug, info, info_span, warn};
use tracing::{instrument::Instrumented, Instrument};

use abacus_core::{chain_from_domain, CommittedMessage, ListValidity, OutboxIndexer};

use crate::{
    contract_sync::{last_message::OptLatestLeafIndex, schema::OutboxContractSyncDB},
    ContractSync,
};

const MESSAGES_LABEL: &str = "messages";

impl<I> ContractSync<I>
where
    I: OutboxIndexer + 'static,
{
    /// Sync outbox messages
    pub fn sync_outbox_messages(&self) -> Instrumented<tokio::task::JoinHandle<eyre::Result<()>>> {
        let span = info_span!("MessageContractSync");

        let db = self.db.clone();
        let indexer = self.indexer.clone();
        let indexed_height = self
            .metrics
            .indexed_height
            .with_label_values(&[MESSAGES_LABEL, &self.chain_name]);

        let stored_messages = self
            .metrics
            .stored_events
            .with_label_values(&[MESSAGES_LABEL, &self.chain_name]);

        let missed_messages = self
            .metrics
            .missed_events
            .with_label_values(&[MESSAGES_LABEL, &self.chain_name]);

        let message_leaf_index = self.metrics.message_leaf_index.clone();
        let chain_name = self.chain_name.clone();

        let config_from = self.index_settings.from();
        let chunk_size = self.index_settings.chunk_size();

        // Indexes messages by fetching messages in ranges of blocks.
        // We've observed occasional flakiness with providers where some events in
        // a range will be missing. The leading theories are:
        // 1. The provider is just flaky and sometimes misses events :(
        // 2. For outbox chains with low finality times, it's possible that when
        //    we query the RPC provider for the latest finalized block number,
        //    we're returned a block number T. However when we attempt to index a range
        //    where the `to` block is T, the `eth_getLogs` RPC is load balanced by the
        //    provider to a different node whose latest known block is some block T' < T.
        //    The `eth_getLogs` RPC implementations seem to happily accept `to` blocks that
        //    exceed the latest known block, so it's possible that in our indexer we think
        //    that we've indexed up to block T but we've only *actually* indexed up to block T'.

        // It's easy to determine if a provider has skipped any message events by
        // looking at the indices of each message and ensuring that we've indexed a valid
        // continuation of messages.
        // There are two classes of invalid continuations:
        // 1. The latest previously indexed message index is M that was found in a previously
        //    indexed block range. A new block range [A,B] is indexed, returning a list of messages.
        //    The lowest message index in that list is `M + 1`, but there are some missing messages
        //    indices in the list. This is likely a flaky provider, and we can simply re-index the
        //    range [A,B] hoping that the provider will soon return a correct list.
        // 2. The latest previously indexed message index is M that was found in a previously
        //    indexed block range, [A,B]. A new block range [C,D] is indexed, returning a list of
        //    messages. However, the lowest message index in that list is M' where M' > M + 1.
        //    This missing messages could be anywhere in the range [A,D]:
        //    * It's possible there was an issue when the prior block range [A,B] was indexed, where
        //      the provider didn't provide some messages with indices > M that it should have.
        //    * It's possible that the range [B,C] that was presumed to be empty when it was indexed
        //      actually wasn't.
        //    * And it's possible that this was just a flaky gap, where there are messages in the [C,D]
        //      range that weren't returned for some reason.
        //    We can handle this by re-indexing starting from block A.
        //    Note this means we only handle this case upon observing messages in some range [C,D]
        //    that indicate a previously indexed range may have missed some messages.
        tokio::spawn(async move {
            let mut from = db
                .retrieve_latest_valid_message_range_start_block()
                .unwrap_or(config_from);

            let mut last_valid_range_start_block = from;

            info!(from = from, "[Messages]: resuming indexer from latest valid message range start block");

            loop {
                indexed_height.set(from as i64);

                // Only index blocks considered final.
                // If there's an error getting the block number, just start the loop over
                let tip = if let Ok(num) = indexer.get_finalized_block_number().await {
                    num
                } else {
                    continue;
                };
                if tip <= from {
                    // Sleep if caught up to tip
                    sleep(Duration::from_secs(1)).await;
                    continue;
                }

                // Index the chunk_size, capping at the tip.
                let to = min(tip, from + chunk_size);

                // Still search the full-size chunk size to possibly catch events that nodes have dropped "close to the tip"
                let full_chunk_from = to.checked_sub(chunk_size).unwrap_or_default();

                let mut sorted_messages = indexer.fetch_sorted_messages(full_chunk_from, to).await?;

                info!(
                    from = full_chunk_from,
                    to = to,
                    message_count = sorted_messages.len(),
                    "[Messages]: indexed block range"
                );

                // Get the latest known leaf index. All messages whose indices are <= this index
                // have been stored in the DB.
                let last_leaf_index: OptLatestLeafIndex = db.retrieve_latest_leaf_index()?.into();

                // Filter out any messages that have already been successfully indexed and stored.
                // This is necessary if we're re-indexing blocks in hope of finding missing messages.
                if let Some(min_index) = last_leaf_index.as_ref() {
                    sorted_messages = sorted_messages.into_iter().filter(|m| m.leaf_index > *min_index).collect();
                }

                debug!(
                    from = full_chunk_from,
                    to = to,
                    message_count = sorted_messages.len(),
                    "[Messages]: filtered any messages already indexed"
                );

                // Continue if no messages found.
                // We don't update last_valid_range_start_block because we cannot extrapolate
                // if the range was correctly indexed if there are no messages to observe their
                // indices.
                if sorted_messages.is_empty() {
                    from = to + 1;
                    continue;
                }

                // Ensure the sorted messages are a valid continuation of last_leaf_index
                match &last_leaf_index.valid_continuation(&sorted_messages) {
                    ListValidity::Valid => {
                        // Store messages
                        let max_leaf_index_of_batch = db.store_messages(&sorted_messages)?;

                        // Report amount of messages stored into db
                        stored_messages.add(sorted_messages.len().try_into()?);

                        // Report latest leaf index to gauge by dst
                        for raw_msg in sorted_messages.iter() {
                            let dst = CommittedMessage::try_from(raw_msg)
                                .ok()
                                .and_then(|msg| chain_from_domain(msg.message.destination))
                                .unwrap_or("unknown");
                            message_leaf_index
                                .with_label_values(&["dispatch", &chain_name, dst])
                                .set(max_leaf_index_of_batch as i64);
                        }

                        // Update the latest valid start block.
                        db.store_latest_valid_message_range_start_block(full_chunk_from)?;
                        last_valid_range_start_block = full_chunk_from;

                        // Move forward to the next height
                        from = to + 1;
                    }
                    // The index of the first message in sorted_messages is not the
                    // `last_leaf_index+1`.
                    ListValidity::InvalidContinuation => {
                        missed_messages.inc();

                        warn!(
                            last_leaf_index = ?last_leaf_index,
                            start_block = from,
                            end_block = to,
                            last_valid_range_start_block,
                            "[Messages]: Found invalid continuation in range. Re-indexing from the start block of the last successful range.",
                        );

                        from = last_valid_range_start_block;
                    }
                    ListValidity::ContainsGaps => {
                        missed_messages.inc();

                        warn!(
                            last_leaf_index = ?last_leaf_index,
                            start_block = from,
                            end_block = to,
                            "[Messages]: Found gaps in the messages in range, re-indexing the same range.",
                        );
                    }
                    ListValidity::Empty => unreachable!("Tried to validate empty list of messages"),
                };
            }
        })
            .instrument(span)
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;
    use std::time::Duration;

    use ethers::core::types::H256;
    use eyre::eyre;
    use mockall::*;
    use tokio::select;
    use tokio::time::{interval, timeout};

    use abacus_core::{db::AbacusDB, AbacusMessage, Encode, RawCommittedMessage};
    use abacus_test::mocks::indexer::MockAbacusIndexer;
    use abacus_test::test_utils;
    use mockall::predicate::eq;

    use crate::contract_sync::schema::OutboxContractSyncDB;
    use crate::ContractSync;
    use crate::{settings::IndexSettings, ContractSyncMetrics, CoreMetrics};

    #[tokio::test]
    async fn handles_missing_rpc_messages() {
        test_utils::run_test_db(|db| async move {
            let mut message_vec = vec![];
            AbacusMessage {
                origin: 1000,
                destination: 2000,
                sender: H256::from([10; 32]),
                recipient: H256::from([11; 32]),
                body: [10u8; 5].to_vec(),
            }
            .write_to(&mut message_vec)
            .expect("!write_to");

            let m0 = RawCommittedMessage {
                leaf_index: 0,
                message: message_vec.clone(),
            };

            let m1 = RawCommittedMessage {
                leaf_index: 1,
                message: message_vec.clone(),
            };

            let m2 = RawCommittedMessage {
                leaf_index: 2,
                message: message_vec.clone(),
            };

            let m3 = RawCommittedMessage {
                leaf_index: 3,
                message: message_vec.clone(),
            };

            let m4 = RawCommittedMessage {
                leaf_index: 4,
                message: message_vec.clone(),
            };

            let m5 = RawCommittedMessage {
                leaf_index: 5,
                message: message_vec.clone(),
            };

            let latest_valid_message_range_start_block = 100;

            let mut mock_indexer = MockAbacusIndexer::new();
            {
                let mut seq = Sequence::new();

                // Return m0.
                let m0_clone = m0.clone();
                mock_indexer
                    .expect__get_finalized_block_number()
                    .times(1)
                    .in_sequence(&mut seq)
                    .return_once(|| Ok(110));
                mock_indexer
                    .expect__fetch_sorted_messages()
                    .times(1)
                    .with(eq(91), eq(110))
                    .in_sequence(&mut seq)
                    .return_once(move |_, _| Ok(vec![m0_clone]));

                // Return m1, miss m2.
                let m1_clone = m1.clone();
                mock_indexer
                    .expect__get_finalized_block_number()
                    .times(1)
                    .in_sequence(&mut seq)
                    .return_once(|| Ok(120));
                mock_indexer
                    .expect__fetch_sorted_messages()
                    .times(1)
                    .with(eq(101), eq(120))
                    .in_sequence(&mut seq)
                    .return_once(move |_, _| Ok(vec![m1_clone]));

                // Miss m3.
                mock_indexer
                    .expect__get_finalized_block_number()
                    .times(1)
                    .in_sequence(&mut seq)
                    .return_once(|| Ok(130));
                mock_indexer
                    .expect__fetch_sorted_messages()
                    .times(1)
                    .with(eq(111), eq(130))
                    .in_sequence(&mut seq)
                    .return_once(move |_, _| Ok(vec![]));

                // Empty range.
                mock_indexer
                    .expect__get_finalized_block_number()
                    .times(1)
                    .in_sequence(&mut seq)
                    .return_once(|| Ok(140));
                mock_indexer
                    .expect__fetch_sorted_messages()
                    .times(1)
                    .with(eq(121), eq(140))
                    .in_sequence(&mut seq)
                    .return_once(move |_, _| Ok(vec![]));

                mock_indexer
                    .expect__get_finalized_block_number()
                    .times(1)
                    .in_sequence(&mut seq)
                    .return_once(|| Ok(140));

                // m1 --> m5 seen as an invalid continuation
                let m5_clone = m5.clone();
                mock_indexer
                    .expect__get_finalized_block_number()
                    .times(1)
                    .in_sequence(&mut seq)
                    .return_once(|| Ok(150));
                mock_indexer
                    .expect__fetch_sorted_messages()
                    .times(1)
                    .with(eq(131), eq(150))
                    .in_sequence(&mut seq)
                    .return_once(move |_, _| Ok(vec![m5_clone]));

                // Indexer goes back to the last valid message range start block
                // and indexes the range based off the chunk size of 19.
                // This time it gets m1 and m2 (which was previously skipped)
                let m1_clone = m1.clone();
                let m2_clone = m2.clone();
                mock_indexer
                    .expect__get_finalized_block_number()
                    .times(1)
                    .in_sequence(&mut seq)
                    .return_once(|| Ok(160));
                mock_indexer
                    .expect__fetch_sorted_messages()
                    .times(1)
                    .with(eq(101), eq(120))
                    .in_sequence(&mut seq)
                    .return_once(move |_, _| Ok(vec![m1_clone, m2_clone]));

                // Indexer continues, this time getting m3 and m5 message, but skipping m4,
                // which means this range contains gaps
                let m3_clone = m3.clone();
                let m5_clone = m5.clone();
                mock_indexer
                    .expect__get_finalized_block_number()
                    .times(1)
                    .in_sequence(&mut seq)
                    .return_once(|| Ok(170));
                mock_indexer
                    .expect__fetch_sorted_messages()
                    .times(1)
                    .with(eq(121), eq(140))
                    .in_sequence(&mut seq)
                    .return_once(move |_, _| Ok(vec![m3_clone, m5_clone]));

                // Indexer retries, the same range in hope of filling the gap,
                // which it now does successfully
                mock_indexer
                    .expect__get_finalized_block_number()
                    .times(1)
                    .in_sequence(&mut seq)
                    .return_once(|| Ok(170));
                mock_indexer
                    .expect__fetch_sorted_messages()
                    .times(1)
                    .with(eq(121), eq(140))
                    .in_sequence(&mut seq)
                    .return_once(move |_, _| Ok(vec![m3, m4, m5]));

                // Indexer continues with the next block range, which happens to be empty
                mock_indexer
                    .expect__get_finalized_block_number()
                    .times(1)
                    .in_sequence(&mut seq)
                    .return_once(|| Ok(180));
                mock_indexer
                    .expect__fetch_sorted_messages()
                    .times(1)
                    .with(eq(141), eq(160))
                    .in_sequence(&mut seq)
                    .return_once(move |_, _| Ok(vec![]));

                // Indexer catches up with the tip
                mock_indexer
                    .expect__get_finalized_block_number()
                    .times(1)
                    .in_sequence(&mut seq)
                    .return_once(|| Ok(180));
                mock_indexer
                    .expect__fetch_sorted_messages()
                    .times(1)
                    .with(eq(161), eq(180))
                    .in_sequence(&mut seq)
                    .return_once(move |_, _| Ok(vec![]));

                // Stay at the same tip, so no other fetch_sorted_messages calls are made
                mock_indexer
                    .expect__get_finalized_block_number()
                    .returning(|| Ok(180));
            }

            let abacus_db = AbacusDB::new("outbox_1", db);

            // Set the latest valid message range start block
            abacus_db
                .store_latest_valid_message_range_start_block(
                    latest_valid_message_range_start_block,
                )
                .unwrap();

            let indexer = Arc::new(mock_indexer);
            let metrics = Arc::new(
                CoreMetrics::new("contract_sync_test", None, prometheus::Registry::new())
                    .expect("could not make metrics"),
            );

            let sync_metrics = ContractSyncMetrics::new(metrics);

            let contract_sync = ContractSync::new(
                "outbox_1".into(),
                abacus_db.clone(),
                indexer.clone(),
                IndexSettings {
                    from: Some("0".to_string()),
                    chunk: Some("19".to_string()),
                },
                sync_metrics,
            );

            let sync_task = contract_sync.sync_outbox_messages();
            let test_pass_fut = timeout(Duration::from_secs(30), async move {
                let mut interval = interval(Duration::from_millis(20));
                loop {
                    if abacus_db.message_by_leaf_index(0).expect("!db").is_some()
                        && abacus_db.message_by_leaf_index(1).expect("!db").is_some()
                        && abacus_db.message_by_leaf_index(2).expect("!db").is_some()
                        && abacus_db.message_by_leaf_index(3).expect("!db").is_some()
                        && abacus_db.message_by_leaf_index(4).expect("!db").is_some()
                        && abacus_db.message_by_leaf_index(5).expect("!db").is_some()
                    {
                        break;
                    }
                    interval.tick().await;
                }
            });
            let test_result = select! {
                 err = sync_task => Err(eyre!(
                    "sync task unexpectedly done before test: {:?}", err.unwrap_err())),
                 tests_result = test_pass_fut =>
                   if tests_result.is_ok() { Ok(()) } else { Err(eyre!("timed out")) }
            };
            assert!(test_result.is_ok());
        })
        .await
    }
}
