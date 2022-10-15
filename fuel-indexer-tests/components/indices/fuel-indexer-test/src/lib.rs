extern crate alloc;
use fuel_indexer_macros::indexer;

#[indexer(
    abi = "fuel-indexer-tests/contracts/fuel-indexer-test/out/debug/fuel-indexer-test-abi.json",
    namespace = "fuel_indexer_test",
    identifier = "index1",
    schema = "./../../../assets/fuel_indexer_test.graphql"
)]

mod fuel_indexer_test {

    use fuel_indexer_macros::indexer;

    fn fuel_indexer_test_ping(ping: Ping) {
        Logger::info("fuel_indexer_test_ping handling a Ping event.");

        let entity = PingEntity {
            id: ping.id,
            value: ping.value,
        };

        entity.save();
    }

    #[block]
    fn fuel_indexer_test_blocks(block: BlockData) {
        let blk = BlockEntity {
            id: block.height,
            hash: block.id,
            height: block.height,
            producer: block.producer,
            timestamp: block.time,
        };

        blk.save();

        let input_data = r#"{"foo":"bar"}"#.to_string();

        for (i, _receipts) in block.transactions.iter().enumerate() {
            let tx = TransactionEntity {
                id: i as u64,
                hash: [0u8; 32].into(),
                block: blk.id,
                timestamp: block.time,
                input_data: Jsonb(input_data.clone()),
            };
            tx.save();
        }
    }

    fn fuel_indexer_test_transfer(transfer: Transfer) {
        Logger::info("fuel_indexer_test_transfer handling Transfer event.");

        let entity = TransferEntity {
            id: 1,
            contract_id: transfer.contract_id,
            recipient: transfer.to,
            amount: 1,
            asset_id: transfer.asset_id,
        };

        entity.save();
    }

    fn fuel_indexer_test_log(log: Log) {
        Logger::info("fuel_indexer_test_log handling Log event.");

        let entity = LogEntity {
            id: 1,
            contract_id: log.contract_id,
            ra: log.ra,
            rb: log.rb,
        };

        entity.save();
    }

    fn fuel_indexer_test_logdata(logdata_entity: Pung) {
        Logger::info("fuel_indexer_test_logdata handling Log event.");

        let entity = PungEntity {
            id: 1,
            value: logdata_entity.value,
            is_pung: 1,
        };

        entity.save();
    }

    fn fuel_indexer_test_scriptresult(scriptresult_entity: ScriptResult) {
        Logger::info("fuel_indexer_test_scriptresult handling Log event.");

        let entity = ScriptResultEntity {
            result: scriptresult_entity.result,
            gas_used: scriptresult_entity.gas_used,
        };

        entity.save();
    }
}
