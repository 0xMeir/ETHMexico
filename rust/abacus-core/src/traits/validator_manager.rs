use std::fmt::Debug;

use async_trait::async_trait;
use eyre::Result;

use crate::{
    accumulator::merkle::Proof,
    traits::{ChainCommunicationError, TxOutcome},
    AbacusMessage, MultisigSignedCheckpoint,
};

/// Interface for an InboxValidatorManager
#[async_trait]
pub trait InboxValidatorManager: Send + Sync + Debug {
    /// Process a message with a proof against the provided signed checkpoint
    async fn process(
        &self,
        multisig_signed_checkpoint: &MultisigSignedCheckpoint,
        message: &AbacusMessage,
        proof: &Proof,
    ) -> Result<TxOutcome, ChainCommunicationError>;
}
