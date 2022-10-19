extern crate alloc;
use fuel_indexer_macros::indexer;

#[indexer(manifest = "fuel-indexer-tests/assets/fuel_indexer_test.yaml")]
mod fuel_indexer_test {
    fn fuel_indexer_test_ping(ping: Ping) {
        Logger::info("fuel_indexer_test_ping handling a Ping event.");

        let entity = PingEntity {
            id: ping.id,
            value: ping.value,
        };

        entity.save();
    }

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

    fn fuel_indexer_test_transferout(transferout: TransferOut) {
        Logger::info("fuel_indexer_test_transferout handling TransferOut event.");

        let entity = TransferOutEntity {
            id: 1,
            contract_id: transferout.contract_id,
            recipient: transferout.to,
            amount: 1,
            asset_id: transferout.asset_id,
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

    // fn fuel_indexer_test_logdata(logdata_entity: Pung) {
    //     Logger::info("fuel_indexer_test_logdata handling Log event.");

    //     let entity = PungEntity {
    //         id: 1,
    //         value: logdata_entity.value,
    //         is_pung: 1,
    //     };

    //     entity.save();
    // }

    fn fuel_indexer_test_scriptresult(scriptresult: ScriptResult) {
        Logger::info("fuel_indexer_test_scriptresult handling ScriptResult event.");

        let entity = ScriptResultEntity {
            id: 1,
            result: scriptresult.result,
            gas_used: scriptresult.gas_used,
        };

        entity.save();
    }
}
