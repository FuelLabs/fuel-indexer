extern crate alloc;
use fuel_indexer_macros::indexer;
use fuel_indexer_plugin::prelude::*;

#[indexer(manifest = "packages/fuel-indexer-tests/assets/fuel_indexer_test.yaml")]
mod fuel_indexer_test {

    fn fuel_indexer_test_ping(ping: Ping) {
        Logger::info("fuel_indexer_test_ping handling a Ping event.");

        let entity = PingEntity {
            id: ping.id,
            value: ping.value,
            message: ping.message.to_string(),
        };

        entity.save();
    }

    fn fuel_indexer_test_u16(_ping: Ping) {
        Logger::info("fuel_indexer_test_ping handling a U16 event.");
        let u16entity = U16Entity {
            id: 9999,
            value1: 340282366920938463463374607431768211454, // 2**128-2
            value2: 170141183460469231731687303715884105727, // 2**127-1
        };

        u16entity.save();
    }

    fn fuel_indexer_test_blocks(block_data: BlockData) {
        let block = Block {
            id: first8_bytes_to_u64(block_data.id),
            height: block_data.height,
            timestamp: block_data.time,
        };

        block.save();

        let input_data = r#"{"foo":"bar"}"#.to_string();

        for tx in block_data.transactions.iter() {
            let tx = Tx {
                id: first8_bytes_to_u64(tx.id),
                block: block.id,
                timestamp: block_data.time,
                input_data: Json(input_data.clone()),
            };
            tx.save();
        }
    }

    fn fuel_indexer_test_transfer(transfer: abi::Transfer) {
        Logger::info("fuel_indexer_test_transfer handling Transfer event.");

        let abi::Transfer {
            contract_id,
            to,
            asset_id,
            amount,
            ..
        } = transfer;

        let entity = Transfer {
            id: first8_bytes_to_u64(contract_id),
            contract_id,
            recipient: to,
            amount,
            asset_id,
        };

        entity.save();
    }

    fn fuel_indexer_test_transferout(transferout: abi::TransferOut) {
        Logger::info("fuel_indexer_test_transferout handling TransferOut event.");

        let abi::TransferOut {
            contract_id,
            to,
            asset_id,
            amount,
            ..
        } = transferout;

        let entity = TransferOut {
            id: first8_bytes_to_u64(contract_id),
            contract_id,
            recipient: to,
            amount,
            asset_id,
        };

        entity.save();
    }

    fn fuel_indexer_test_log(log: abi::Log) {
        Logger::info("fuel_indexer_test_log handling Log event.");

        let abi::Log {
            contract_id,
            rb,
            ra,
            ..
        } = log;

        let entity = Log {
            id: first8_bytes_to_u64(contract_id),
            contract_id: log.contract_id,
            ra,
            rb,
        };

        entity.save();
    }

    fn fuel_indexer_test_logdata(logdata_entity: Pung) {
        Logger::info("fuel_indexer_test_logdata handling LogData event.");

        let entity = PungEntity {
            id: logdata_entity.id,
            value: logdata_entity.value,
            is_pung: logdata_entity.is_pung,
            // TODO: https://github.com/FuelLabs/fuel-indexer/issues/386
            pung_from: Identity::from(logdata_entity.pung_from),
        };

        entity.save();
    }

    fn fuel_indexer_test_scriptresult(scriptresult: abi::ScriptResult) {
        Logger::info("fuel_indexer_test_scriptresult handling ScriptResult event.");

        let abi::ScriptResult { result, gas_used } = scriptresult;

        let entity = ScriptResult {
            id: result,
            result,
            gas_used,
            blob: vec![1u8, 1, 1, 1, 1],
        };

        entity.save();
    }

    fn fuel_indexer_test_messageout(messageout: abi::MessageOut) {
        Logger::info("fuel_indexer_test_messageout handling MessageOut event");

        let abi::MessageOut {
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
            id: first8_bytes_to_u64(message_id),
            sender,
            recipient,
            amount,
            nonce,
            len,
            digest,
        };

        entity.save();
    }

    fn fuel_indexer_test_callreturn(pungentity: Pung) {
        Logger::info("fuel_indexer_test_callreturn handling Pung event.");

        let entity = PungEntity {
            id: pungentity.id,
            value: pungentity.value,
            is_pung: pungentity.is_pung,
            // TODO: https://github.com/FuelLabs/fuel-indexer/issues/386
            pung_from: Identity::from(pungentity.pung_from),
        };

        entity.save();
    }

    fn fuel_indexer_test_multiargs(
        pung: Pung,
        pong: Pong,
        ping: Ping,
        block_data: BlockData,
    ) {
        Logger::info("fuel_indexer_test_multiargs handling Pung, Pong, Ping, and BlockData events.");

        let block = Block {
            id: first8_bytes_to_u64(block_data.id),
            height: block_data.height,
            timestamp: block_data.time,
        };

        block.save();

        let pu = PungEntity {
            id: pung.id,
            value: pung.value,
            is_pung: pung.is_pung,
            pung_from: Identity::from(pung.pung_from),
        };

        pu.save();

        let po = PongEntity {
            id: pong.id,
            value: pong.value,
        };

        po.save();

        let pi = PingEntity {
            id: ping.id,
            value: ping.value,
            message: ping.message.to_string(),
        };

        pi.save();
    }

    fn fuel_indexer_test_optional_schema_fields(ping: Ping) {
        Logger::info("fuel_indexer_test_optional_schema_fields handling Ping event and setting optional fields.");

        let entity = OptionEntity {
            id: ping.id,
            int_required: 100,
            int_optional_some: Some(999),
            addr_optional_none: None,
        };

        entity.save();
    }

    fn fuel_indexer_test_tuple(
        event: ComplexTupleStruct,
        logdata_entity: SimpleTupleStruct,
    ) {
        Logger::info("fuel_indexer_test_tuple handling ComplexTupleStruct and SimpleTupleStruct events.");
        let data: (u32, (u64, bool, (SizedAsciiString<5>, TupleStructItem))) = event.data;
        let entity = TupleEntity {
            id: data.1 .0,
            complex_a: data.1 .2 .0.to_string(),
            complex_b: data.1 .2 .1.id,
            simple_a: logdata_entity.data.2.to_string(),
        };
        entity.save();
    }
}
