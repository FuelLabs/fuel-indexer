extern crate alloc;
use fuel_indexer_macros::indexer;
use fuel_indexer_plugin::{types::Bytes32, utils::sha256_digest};

// Copied over from the block-explorer example
pub fn derive_id(id: [u8; 32], data: Vec<u8>) -> Bytes32 {
    let mut buff: [u8; 32] = [0u8; 32];
    let result = [id.to_vec(), data].concat();
    buff.copy_from_slice(&sha256_digest(&result).as_bytes()[..32]);
    Bytes32::from(buff)
}

#[indexer(manifest = "packages/fuel-indexer-tests/assets/fuel_indexer_test.yaml")]
mod fuel_indexer_test {

    fn fuel_indexer_test_ping(ping: Ping) {
        Logger::info("fuel_indexer_test_ping handling a Ping event.");

        let entity = PingEntity {
            id: ping.id,
            value: ping.value,
        };

        entity.save();
    }

    fn fuel_indexer_test_blocks(block_data: BlockData) {
        let block = Block {
            id: block_data.id,
            height: block_data.height,
            timestamp: block_data.time,
        };

        block.save();

        let input_data = r#"{"foo":"bar"}"#.to_string();

        for tx in block_data.transactions.iter() {
            let tx = Tx {
                id: tx.id,
                block: block.id,
                timestamp: block_data.time,
                input_data: Jsonb(input_data.clone()),
            };
            tx.save();
        }
    }

    fn fuel_indexer_test_transfer(transfer: fuel::Transfer) {
        Logger::info("fuel_indexer_test_transfer handling Transfer event.");

        let fuel::Transfer {
            contract_id,
            to,
            asset_id,
            amount,
            ..
        } = transfer;

        let entity = Transfer {
            id: derive_id(
                *contract_id,
                [contract_id.to_vec(), to.to_vec(), asset_id.to_vec()].concat(),
            ),
            contract_id,
            recipient: to,
            amount,
            asset_id,
        };

        entity.save();
    }

    fn fuel_indexer_test_transferout(transferout: fuel::TransferOut) {
        Logger::info("fuel_indexer_test_transferout handling TransferOut event.");

        let fuel::TransferOut {
            contract_id,
            to,
            asset_id,
            amount,
            ..
        } = transferout;

        let entity = TransferOut {
            id: derive_id(
                *contract_id,
                [contract_id.to_vec(), to.to_vec(), asset_id.to_vec()].concat(),
            ),
            contract_id,
            recipient: to,
            amount,
            asset_id,
        };

        entity.save();
    }

    fn fuel_indexer_test_log(log: fuel::Log) {
        Logger::info("fuel_indexer_test_log handling Log event.");

        let fuel::Log {
            contract_id, rb, ..
        } = log;

        let entity = Log {
            id: derive_id(*contract_id, u64::to_le_bytes(rb).to_vec()),
            contract_id: log.contract_id,
            ra: log.ra,
            rb: log.rb,
        };

        entity.save();
    }

    fn fuel_indexer_test_logdata(logdata_entity: Pung) {
        Logger::info("fuel_indexer_test_logdata handling LogData event.");

        let entity = PungEntity {
            id: 1,
            value: logdata_entity.value,
            is_pung: 1,
            from: logdata_entity.from,
        };

        entity.save();
    }

    fn fuel_indexer_test_scriptresult(scriptresult: fuel::ScriptResult) {
        Logger::info("fuel_indexer_test_scriptresult handling ScriptResult event.");

        let fuel::ScriptResult { result, gas_used } = scriptresult;

        let entity = ScriptResult {
            id: derive_id([0u8; 32], u64::to_be_bytes(result).to_vec()),
            result,
            gas_used,
        };

        entity.save();
    }

    fn fuel_indexer_test_messageout(messageout: fuel::MessageOut) {
        Logger::info("fuel_indexer_test_messageout handling MessageOut event");

        let fuel::MessageOut {
            sender,
            message_id,
            recipient,
            amount,
            nonce,
            len,
            digest,
            ..
        } = messageout;

        let entity = MessageOut {
            id: message_id,
            sender,
            recipient,
            amount,
            nonce,
            len,
            digest,
        };

        entity.save();
    }
}
