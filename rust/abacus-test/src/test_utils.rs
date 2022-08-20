use futures_util::Future;
use rocksdb::Options;
use tempfile::TempDir;

use abacus_core::db::DB;

pub fn setup_db(db_path: String) -> DB {
    let mut opts = Options::default();
    opts.create_if_missing(true);
    rocksdb::DB::open(&opts, db_path)
        .expect("Failed to open db path")
        .into()
}

pub async fn run_test_db<T, Fut>(test: T)
where
    T: FnOnce(DB) -> Fut,
    Fut: Future<Output = ()>,
{
    // Use `/tmp`-equivalent so that any resource leak of the db files will
    // eventually be cleaned up, even if e.g. TempDir's drop handler never runs
    // due to a segfault etc encountered during the test.
    let db_tmp_dir = TempDir::new().unwrap();
    let db = setup_db(db_tmp_dir.path().to_str().unwrap().into());
    let _test_result = test(db).await;
    let _ = rocksdb::DB::destroy(&Options::default(), db_tmp_dir);
}

#[cfg(test)]
mod test {
    use ethers::types::H256;

    use abacus_core::{
        accumulator::merkle::Proof, db::AbacusDB, AbacusMessage, Encode, RawCommittedMessage,
    };

    use super::*;

    #[tokio::test]
    async fn db_stores_and_retrieves_messages() {
        run_test_db(|db| async move {
            let outbox_name = "outbox_1".to_owned();
            let db = AbacusDB::new(outbox_name, db);

            let leaf_index = 100;
            let m = AbacusMessage {
                origin: 10,
                sender: H256::from_low_u64_be(4),
                destination: 12,
                recipient: H256::from_low_u64_be(5),
                body: vec![1, 2, 3],
            };

            let message = RawCommittedMessage {
                leaf_index,
                message: m.to_vec(),
            };

            assert_eq!(m.to_leaf(leaf_index), message.leaf());

            db.store_raw_committed_message(&message).unwrap();

            let by_leaf = db.message_by_leaf(message.leaf()).unwrap().unwrap();
            assert_eq!(by_leaf, message);

            let by_index = db
                .message_by_leaf_index(message.leaf_index)
                .unwrap()
                .unwrap();
            assert_eq!(by_index, message);
        })
        .await;
    }

    #[tokio::test]
    async fn db_stores_and_retrieves_proofs() {
        run_test_db(|db| async move {
            let outbox_name = "outbox_1".to_owned();
            let db = AbacusDB::new(outbox_name, db);

            let proof = Proof {
                leaf: H256::from_low_u64_be(15),
                index: 32,
                path: Default::default(),
            };
            db.store_proof(13, &proof).unwrap();

            let by_index = db.proof_by_leaf_index(13).unwrap().unwrap();
            assert_eq!(by_index, proof);
        })
        .await;
    }
}
